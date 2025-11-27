use chrono::Utc;

use crate::{
    config::{BanCookie, ClewdrCookie},
    db::{CookieStatus, Database, NewCookie, Queries, UpdateCookie},
    error::ClewdrError,
};
use tracing::{error, info, warn};

#[derive(Debug, Clone, serde::Serialize)]
pub struct BanQueueInfo {
    pub pending: Vec<BanCookie>,
    pub processing: Vec<BanCookie>,
    pub banned: Vec<BanCookie>,
    pub total_requests: u64,
}

#[derive(Debug, Clone)]
pub struct BanQueueHandle {
    db: Database,
}

impl BanQueueHandle {
    pub async fn start_with_db(db: Database) -> Result<Self, ClewdrError> {
        Ok(Self { db })
    }

    fn pool(&self) -> &sqlx::SqlitePool {
        self.db.pool()
    }

    pub async fn submit(&self, cookie: BanCookie) -> Result<(), ClewdrError> {
        let new_cookie = NewCookie {
            cookie: cookie.cookie.to_string(),
            status: CookieStatus::Pending,
        };
        Queries::create_cookie(self.pool(), new_cookie).await?;
        Ok(())
    }

    pub async fn pop(&self) -> Result<BanCookie, ClewdrError> {
        if let Some(db_cookie) = Queries::pop_pending_cookie(self.pool()).await? {
            return Ok(BanCookie {
                cookie: db_cookie.cookie.parse()?,
                submitted_at: Some(db_cookie.created_at.to_rfc3339()),
                last_used_at: db_cookie.last_used.map(|dt| dt.to_rfc3339()),
                requests_sent: db_cookie.request_count as u64,
                is_banned: matches!(db_cookie.status, CookieStatus::Banned),
                error_message: db_cookie.error_message,
            });
        }
        Err(ClewdrError::NoCookieAvailable)
    }

    pub async fn mark_processed(
        &self,
        cookie: String,
        banned: bool,
        error_message: Option<String>,
        next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
        last_rate_limited_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), ClewdrError> {
        self.mark_processed_internal(
            cookie,
            banned,
            error_message,
            next_retry_at,
            last_rate_limited_at,
        )
        .await
    }

    async fn mark_processed_internal(
        &self,
        cookie: String,
        banned: bool,
        error_message: Option<String>,
        next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
        last_rate_limited_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), ClewdrError> {
        if let Some(db_cookie) = Queries::get_cookie_by_value(self.pool(), &cookie).await? {
            let updates = UpdateCookie {
                status: Some(if banned {
                    CookieStatus::Banned
                } else {
                    CookieStatus::Pending
                }),
                last_used: Some(Utc::now()),
                request_count: Some(db_cookie.request_count.saturating_add(1)),
                next_retry_at,
                last_rate_limited_at,
                error_message,
            };
            let _ = Queries::update_cookie(self.pool(), db_cookie.id, updates).await?;
            info!(
                "mark_processed cookie={} banned={} next_retry_at={:?}",
                cookie, banned, next_retry_at
            );
        }
        Ok(())
    }

    /// 写库带重试，避免瞬时 DB Busy/锁冲突
    pub async fn mark_processed_with_retry(
        &self,
        cookie: String,
        banned: bool,
        error_message: Option<String>,
        next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
        last_rate_limited_at: Option<chrono::DateTime<chrono::Utc>>,
        max_retries: u32,
    ) -> Result<(), ClewdrError> {
        let mut attempt = 1;
        loop {
            match self
                .mark_processed_internal(
                    cookie.clone(),
                    banned,
                    error_message.clone(),
                    next_retry_at,
                    last_rate_limited_at,
                )
                .await
            {
                Ok(_) => return Ok(()),
                Err(e) if attempt < max_retries => {
                    warn!(
                        "mark_processed retry {}/{} failed: {}",
                        attempt, max_retries, e
                    );
                    attempt += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub async fn delete(&self, cookie: String) -> Result<(), ClewdrError> {
        Queries::delete_cookie(self.pool(), &cookie).await?;
        Ok(())
    }

    pub async fn get_status(&self) -> Result<BanQueueInfo, ClewdrError> {
        let queue_info = Queries::get_queue_info(self.pool()).await?;

        let pending = queue_info
            .pending
            .into_iter()
            .map(|c| Self::map_cookie_record(c, false))
            .collect();

        let processing = queue_info
            .processing
            .into_iter()
            .map(|c| Self::map_cookie_record(c, false))
            .collect();

        let banned = queue_info
            .banned
            .into_iter()
            .map(|c| Self::map_cookie_record(c, true))
            .collect();

        Ok(BanQueueInfo {
            pending,
            processing,
            banned,
            total_requests: queue_info.total_requests as u64,
        })
    }

    pub async fn reset_stats(&self) -> Result<(), ClewdrError> {
        Queries::reset_stats(self.pool()).await?;
        sqlx::query("UPDATE cookies SET request_count = 0")
            .execute(self.pool())
            .await?;
        Ok(())
    }

    pub async fn clear_all(&self) -> Result<(), ClewdrError> {
        Queries::clear_pending(self.pool()).await?;
        Queries::clear_banned(self.pool()).await?;
        Ok(())
    }

    fn map_cookie_record(c: crate::db::Cookie, is_banned: bool) -> BanCookie {
        match c.cookie.parse::<ClewdrCookie>() {
            Ok(parsed) => BanCookie {
                cookie: parsed,
                submitted_at: Some(c.created_at.to_rfc3339()),
                last_used_at: c.last_used.map(|dt| dt.to_rfc3339()),
                requests_sent: c.request_count as u64,
                is_banned,
                error_message: c.error_message,
            },
            Err(e) => {
                error!("Stored cookie '{}' failed to parse: {}", c.cookie, e);
                // default() 带有占位字符串，记录错误方便前端展示
                BanCookie {
                    cookie: ClewdrCookie::default(),
                    submitted_at: Some(c.created_at.to_rfc3339()),
                    last_used_at: c.last_used.map(|dt| dt.to_rfc3339()),
                    requests_sent: c.request_count as u64,
                    is_banned,
                    error_message: Some(
                        c.error_message
                            .unwrap_or_else(|| "invalid_cookie_format".to_string()),
                    ),
                }
            }
        }
    }
}
