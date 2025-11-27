use super::models::*;
use crate::error::ClewdrError;
use crate::middleware::{validate_config_key, validate_config_value};
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

pub struct Queries;

impl Queries {
    // Cookie queries
    pub async fn create_cookie(
        pool: &SqlitePool,
        cookie: NewCookie,
    ) -> Result<Cookie, ClewdrError> {
        let now = Utc::now();

        let row = sqlx::query_as::<_, Cookie>(
            r#"
            INSERT INTO cookies (cookie, status, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            RETURNING
              id,
              cookie,
              status as "status: CookieStatus",
              created_at,
              updated_at,
              last_used,
              next_retry_at,
              last_rate_limited_at,
              request_count,
              error_message
            "#,
        )
        .bind(&cookie.cookie)
        .bind(cookie.status.to_string())
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn get_cookie_by_value(
        pool: &SqlitePool,
        cookie_str: &str,
    ) -> Result<Option<Cookie>, ClewdrError> {
        let cookie = sqlx::query_as::<_, Cookie>(
            r#"
            SELECT
                id, cookie, status as "status: CookieStatus",
                created_at, updated_at, last_used,
                next_retry_at, last_rate_limited_at,
                request_count, error_message
            FROM cookies
            WHERE cookie = ?1
            "#,
        )
        .bind(cookie_str)
        .fetch_optional(pool)
        .await?;

        Ok(cookie)
    }

    pub async fn update_cookie(
        pool: &SqlitePool,
        id: i64,
        updates: UpdateCookie,
    ) -> Result<Cookie, ClewdrError> {
        let now = Utc::now();
        let row = sqlx::query_as::<_, Cookie>(
            r#"
            UPDATE cookies
            SET
                status = COALESCE(?1, status),
                last_used = COALESCE(?2, last_used),
                request_count = COALESCE(?3, request_count),
                next_retry_at = COALESCE(?4, next_retry_at),
                last_rate_limited_at = COALESCE(?5, last_rate_limited_at),
                error_message = ?6,
                updated_at = ?7
            WHERE id = ?8
            RETURNING
                id,
                cookie,
                status as "status: CookieStatus",
                created_at,
                updated_at,
                last_used,
                next_retry_at,
                last_rate_limited_at,
                request_count,
                error_message
            "#,
        )
        .bind(updates.status.as_ref().map(|s| s.to_string()))
        .bind(updates.last_used)
        .bind(updates.request_count)
        .bind(updates.next_retry_at)
        .bind(updates.last_rate_limited_at)
        .bind(updates.error_message)
        .bind(now)
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    pub async fn delete_cookie(pool: &SqlitePool, cookie_str: &str) -> Result<bool, ClewdrError> {
        let result = sqlx::query!("DELETE FROM cookies WHERE cookie = ?", cookie_str)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_cookies_by_status(
        pool: &SqlitePool,
        status: CookieStatus,
    ) -> Result<Vec<Cookie>, ClewdrError> {
        let status_str = status.to_string();
        let (order_by, order_dir) = match status {
            CookieStatus::Pending => ("created_at", "ASC"),
            _ => ("updated_at", "DESC"),
        };

        let sql = format!(
            r#"
            SELECT
                id, cookie, status as "status: CookieStatus",
                created_at, updated_at, last_used,
                next_retry_at, last_rate_limited_at,
                request_count, error_message
            FROM cookies
            WHERE status = ?
            ORDER BY {order_by} {order_dir}
            "#
        );

        Ok(sqlx::query_as::<_, Cookie>(&sql)
            .bind(status_str)
            .fetch_all(pool)
            .await?)
    }

    pub async fn pop_pending_cookie(pool: &SqlitePool) -> Result<Option<Cookie>, ClewdrError> {
        let cookie = sqlx::query_as::<_, Cookie>(
            r#"
            UPDATE cookies
            SET status = 'checking', updated_at = ?1
            WHERE id = (
                SELECT id FROM cookies
                WHERE status = 'pending'
                  AND (next_retry_at IS NULL OR next_retry_at <= ?1)
                ORDER BY created_at ASC
                LIMIT 1
            )
            RETURNING
                id, cookie, status as "status: CookieStatus",
                created_at, updated_at, last_used,
                next_retry_at, last_rate_limited_at,
                request_count, error_message
            "#,
        )
        .bind(Utc::now())
        .fetch_optional(pool)
        .await?;
        Ok(cookie)
    }

    pub async fn expire_old_pending(
        pool: &SqlitePool,
        older_than_days: i64,
    ) -> Result<u64, ClewdrError> {
        let cutoff = Utc::now() - chrono::Duration::days(older_than_days);
        let result = sqlx::query!(
            r#"
            UPDATE cookies
            SET status = 'banned',
                error_message = 'expired_after_48h',
                updated_at = ?1
            WHERE status = 'pending'
              AND created_at < ?2
            "#,
            cutoff,
            cutoff
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_queue_info(pool: &SqlitePool) -> Result<QueueInfo, ClewdrError> {
        let pending = Self::get_cookies_by_status(pool, CookieStatus::Pending).await?;
        let processing = Self::get_cookies_by_status(pool, CookieStatus::Checking).await?;
        let banned = Self::get_cookies_by_status(pool, CookieStatus::Banned).await?;

        let total_requests: Option<i64> =
            sqlx::query_scalar("SELECT SUM(request_count) FROM cookies")
                .fetch_one(pool)
                .await?;

        Ok(QueueInfo {
            pending,
            processing,
            banned,
            total_requests: total_requests.unwrap_or(0),
        })
    }

    pub async fn get_aggregated_stats(pool: &SqlitePool) -> Result<AggregatedStats, ClewdrError> {
        let row = sqlx::query_as!(
            AggregatedStats,
            r#"
            SELECT 
                COUNT(*) as total_cookies,
                COALESCE(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END), 0) as pending_count,
                COALESCE(SUM(CASE WHEN status = 'banned' THEN 1 ELSE 0 END), 0) as banned_count,
                COALESCE(SUM(request_count), 0) as total_requests,
                COALESCE(AVG(request_count), 0) as "avg_requests_per_cookie!"
            FROM cookies
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    // Stats queries
    pub async fn create_stats(pool: &SqlitePool, stats: NewStats) -> Result<Stats, ClewdrError> {
        let row = sqlx::query_as::<_, Stats>(
            r#"
            INSERT INTO stats (total_requests, success_count, error_count, avg_response_time)
            VALUES (?1, ?2, ?3, ?4)
            RETURNING id, timestamp, total_requests, success_count, error_count, avg_response_time
            "#,
        )
        .bind(stats.total_requests)
        .bind(stats.success_count)
        .bind(stats.error_count)
        .bind(stats.avg_response_time)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn get_recent_stats(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<Stats>, ClewdrError> {
        let stats = sqlx::query_as::<_, Stats>(
            r#"
            SELECT id, timestamp, total_requests, success_count, error_count, avg_response_time
            FROM stats
            ORDER BY timestamp DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(stats)
    }

    pub async fn get_stats_between(
        pool: &SqlitePool,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Stats>, ClewdrError> {
        let stats = sqlx::query_as::<_, Stats>(
            r#"
            SELECT id, timestamp, total_requests, success_count, error_count, avg_response_time
            FROM stats
            WHERE timestamp BETWEEN ?1 AND ?2
            ORDER BY timestamp ASC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await?;

        Ok(stats)
    }

    // Config queries
    pub async fn upsert_config(
        pool: &SqlitePool,
        key: &str,
        value: &str,
    ) -> Result<Config, ClewdrError> {
        let now = Utc::now();

        // Validate key and value before insertion
        let validated_key = validate_config_key(key)?;
        let validated_value = validate_config_value(value)?;

        let config = sqlx::query_as::<_, Config>(
            r#"
            INSERT OR REPLACE INTO config (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            RETURNING key, value, updated_at
            "#,
        )
        .bind(validated_key)
        .bind(validated_value)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(config)
    }

    pub async fn get_config(pool: &SqlitePool, key: &str) -> Result<Option<String>, ClewdrError> {
        let validated_key = validate_config_key(key)?;
        let value = sqlx::query_scalar!("SELECT value FROM config WHERE key = ?", validated_key)
            .fetch_optional(pool)
            .await?;

        Ok(value)
    }

    // Cleanup queries
    pub async fn clear_pending(pool: &SqlitePool) -> Result<u64, ClewdrError> {
        let result = sqlx::query!("DELETE FROM cookies WHERE status = 'pending'")
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn clear_banned(pool: &SqlitePool) -> Result<u64, ClewdrError> {
        let result = sqlx::query!("DELETE FROM cookies WHERE status = 'banned'")
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn cleanup_old_cookies(pool: &SqlitePool, days: i64) -> Result<u64, ClewdrError> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let result = sqlx::query!(
            "DELETE FROM cookies WHERE status = 'banned' AND updated_at < ?",
            cutoff
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn reset_stats(pool: &SqlitePool) -> Result<(), ClewdrError> {
        sqlx::query!("DELETE FROM stats").execute(pool).await?;

        Ok(())
    }

    // Batch operations for better performance
    pub async fn create_multiple_cookies(
        pool: &SqlitePool,
        cookies: Vec<NewCookie>,
    ) -> Result<Vec<Cookie>, ClewdrError> {
        if cookies.is_empty() {
            return Ok(vec![]);
        }

        let now = Utc::now();
        let mut builder = QueryBuilder::<Sqlite>::new(
            "INSERT INTO cookies (cookie, status, created_at, updated_at) ",
        );

        builder.push_values(cookies.iter(), |mut b, cookie| {
            b.push_bind(&cookie.cookie)
                .push_bind(cookie.status.to_string())
                .push_bind(now)
                .push_bind(now);
        });

        builder.push(
            " RETURNING id, cookie, status as \"status: CookieStatus\", created_at, updated_at, last_used, next_retry_at, last_rate_limited_at, request_count, error_message",
        );

        let result = builder.build_query_as::<Cookie>().fetch_all(pool).await?;

        Ok(result)
    }
}
