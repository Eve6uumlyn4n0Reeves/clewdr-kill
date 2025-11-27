use std::sync::Arc;
use tokio::{
    signal,
    time::{self, Duration},
};
use tracing::{info, warn};

use crate::{db::Database, error::ClewdrError};

/// 数据库清理服务
/// 负责定期清理过期的banned cookie和统计数据,防止数据库膨胀
pub struct CleanupService {
    db: Database,
}

impl CleanupService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 启动清理服务,返回任务句柄
    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        let service = Arc::new(self);

        tokio::spawn(async move {
            info!("Cleanup service started");

            // 每小时执行一次清理
            let mut interval = time::interval(Duration::from_secs(3600));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = service.run_cleanup().await {
                            warn!("Cleanup task failed: {}", e);
                        }
                    }
                    _ = signal::ctrl_c() => {
                        info!("Cleanup service shutting down");
                        break;
                    }
                }
            }
        })
    }

    /// 执行清理任务
    async fn run_cleanup(&self) -> Result<(), ClewdrError> {
        let pool = self.db.pool();

        // 1. 清理7天前的banned cookie
        let banned_deleted = crate::db::Queries::cleanup_old_cookies(pool, 7).await?;
        if banned_deleted > 0 {
            info!(
                audit = true,
                action = "cleanup_old_cookies",
                deleted_count = banned_deleted,
                "Cleaned up {} old banned cookies",
                banned_deleted
            );
        }

        // 2. 清理超过48小时仍pending的cookie (视为过期)
        let expired_count = crate::db::Queries::expire_old_pending(pool, 2).await?;
        if expired_count > 0 {
            info!(
                audit = true,
                action = "expire_old_pending",
                expired_count = expired_count,
                "Expired {} old pending cookies (>48h)",
                expired_count
            );
        }

        // 3. 清理30天前的统计数据
        let stats_deleted = self.cleanup_old_stats(30).await?;
        if stats_deleted > 0 {
            info!(
                audit = true,
                action = "cleanup_old_stats",
                deleted_count = stats_deleted,
                "Cleaned up {} old statistics records",
                stats_deleted
            );
        }

        // 4. 执行VACUUM优化数据库 (每次清理后)
        self.vacuum_database().await?;

        Ok(())
    }

    /// 清理旧的统计数据
    async fn cleanup_old_stats(&self, days: i64) -> Result<u64, ClewdrError> {
        let pool = self.db.pool();
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);

        let result = sqlx::query!("DELETE FROM stats WHERE timestamp < ?", cutoff)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// 执行VACUUM优化数据库文件大小
    async fn vacuum_database(&self) -> Result<(), ClewdrError> {
        let pool = self.db.pool();

        // VACUUM不能在事务中执行,需要单独连接
        sqlx::query("VACUUM").execute(pool).await.map_err(|e| {
            warn!("VACUUM failed (non-critical): {}", e);
            e
        })?;

        info!("Database VACUUM completed");
        Ok(())
    }

    /// 手动触发清理 (用于管理员操作)
    pub async fn manual_cleanup(&self) -> Result<CleanupResult, ClewdrError> {
        let pool = self.db.pool();

        let banned_deleted = crate::db::Queries::cleanup_old_cookies(pool, 7).await?;
        let expired_count = crate::db::Queries::expire_old_pending(pool, 2).await?;
        let stats_deleted = self.cleanup_old_stats(30).await?;

        info!(
            audit = true,
            action = "manual_cleanup",
            banned_deleted = banned_deleted,
            expired_count = expired_count,
            stats_deleted = stats_deleted,
            "Manual cleanup completed"
        );

        Ok(CleanupResult {
            banned_deleted,
            expired_count,
            stats_deleted,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CleanupResult {
    pub banned_deleted: u64,
    pub expired_count: u64,
    pub stats_deleted: u64,
}
