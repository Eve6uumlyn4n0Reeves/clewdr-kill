//! Database connection pool configuration
//! Provides optimized settings for SQLite connection pooling

use crate::error::ClewdrError;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::time::Duration;

/// Create optimized SQLite connection pool
pub async fn create_pool(database_url: &str) -> Result<SqlitePool, ClewdrError> {
    // Configure connection pool for optimal performance
    let pool_options = SqlitePoolOptions::new()
        .max_connections(20) // Increased for concurrent operations
        .min_connections(2) // Keep minimum connections alive
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .test_before_acquire(true);

    let pool = pool_options.connect(database_url).await.map_err(|e| {
        tracing::error!("Failed to create database pool: {}", e);
        ClewdrError::DatabaseError {
            msg: format!("Database connection failed: {}", e),
        }
    })?;

    // Configure SQLite pragmas for better performance
    configure_sqlite_settings(&pool).await?;

    tracing::info!("Database pool created successfully");
    Ok(pool)
}

/// Configure SQLite settings for optimal performance
async fn configure_sqlite_settings(pool: &SqlitePool) -> Result<(), ClewdrError> {
    // Enable WAL mode for better concurrent access
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enable WAL mode: {}", e);
            ClewdrError::DatabaseError {
                msg: format!("Failed to configure WAL mode: {}", e),
            }
        })?;

    // Optimize for concurrent writes
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set synchronous mode: {}", e);
            ClewdrError::DatabaseError {
                msg: format!("Failed to configure synchronous mode: {}", e),
            }
        })?;

    // Increase cache size (default is 2MB, increase to 10MB)
    sqlx::query("PRAGMA cache_size = -10000") // Negative value means KB
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set cache size: {}", e);
            ClewdrError::DatabaseError {
                msg: format!("Failed to configure cache size: {}", e),
            }
        })?;

    // Enable memory-mapped I/O for better performance
    if let Err(e) = sqlx::query("PRAGMA mmap_size = 268435456")
        .execute(pool)
        .await
    {
        tracing::warn!("Failed to enable memory-mapped I/O: {}", e);
    }

    // Optimize temp store
    if let Err(e) = sqlx::query("PRAGMA temp_store = memory")
        .execute(pool)
        .await
    {
        tracing::warn!("Failed to set temp store: {}", e);
    }

    // Configure busy timeout for better handling of concurrent access
    sqlx::query("PRAGMA busy_timeout = 30000") // 30 seconds
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to set busy timeout: {}", e);
            ClewdrError::DatabaseError {
                msg: format!("Failed to configure busy timeout: {}", e),
            }
        })?;

    // Enable foreign key constraints
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to enable foreign keys: {}", e);
            ClewdrError::DatabaseError {
                msg: format!("Failed to enable foreign keys: {}", e),
            }
        })?;

    tracing::info!("SQLite settings configured successfully");
    Ok(())
}

/// Run database maintenance operations
pub async fn run_maintenance(pool: &SqlitePool) -> Result<(), ClewdrError> {
    tracing::info!("Running database maintenance");

    // Analyze the database to update query planner statistics
    sqlx::query("ANALYZE").execute(pool).await.map_err(|e| {
        tracing::error!("Failed to analyze database: {}", e);
        ClewdrError::DatabaseError {
            msg: format!("Database analysis failed: {}", e),
        }
    })?;

    // Vacuum the database to reclaim space (run less frequently)
    // Note: Vacuum can be expensive, so we'll run it conditionally
    let size_check = sqlx::query_scalar!(
        "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()"
    )
    .fetch_one(pool)
    .await;

    if let Ok(Some(size)) = size_check {
        // If database is larger than 100MB, consider vacuuming
        if size > 100 * 1024 * 1024 {
            tracing::info!("Database size is {} bytes, running VACUUM", size);
            sqlx::query("VACUUM").execute(pool).await.map_err(|e| {
                tracing::error!("Failed to vacuum database: {}", e);
                ClewdrError::DatabaseError {
                    msg: format!("Database vacuum failed: {}", e),
                }
            })?;
        }
    }

    // Checkpoint the WAL file to control its size
    if let Err(e) = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(pool)
        .await
    {
        tracing::warn!("Failed to checkpoint WAL: {}", e);
    }

    tracing::info!("Database maintenance completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        // This test would require an in-memory database
        let url = "sqlite::memory:";
        let pool = create_pool(url).await;
        assert!(pool.is_ok());
    }
}
