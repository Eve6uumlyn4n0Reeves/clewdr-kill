use super::pool_config::create_pool;
use crate::{config::CONFIG_PATH, error::ClewdrError};
use sqlx::SqlitePool;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, ClewdrError> {
        let db_path = std::env::var("DATABASE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                CONFIG_PATH
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .join("clewdr.db")
            });

        let database_url = format!("sqlite:{}", db_path.display());

        let pool = create_pool(&database_url).await?;

        let db = Self { pool };

        // Run migrations
        db.migrate().await?;

        Ok(db)
    }

    pub async fn migrate(&self) -> Result<(), ClewdrError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| ClewdrError::DatabaseError {
                msg: format!("Migration failed: {}", e),
            })?;
        tracing::info!("Database migration completed successfully");
        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}
