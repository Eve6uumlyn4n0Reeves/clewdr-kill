// 测试环境搭建工具

use sqlx::SqlitePool;
use std::path::PathBuf;
use tempfile::TempDir;

/// 测试上下文，包含测试所需的所有资源
pub struct TestContext {
    pub db_pool: SqlitePool,
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
}

impl TestContext {
    /// 创建新的测试上下文
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let db_url = format!("sqlite:{}", db_path.display());
        let db_pool = SqlitePool::connect(&db_url)
            .await
            .expect("Failed to connect to test database");

        // 运行迁移
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("Failed to run migrations");

        let config_path = temp_dir.path().join("test_config.toml");

        Self {
            db_pool,
            temp_dir,
            config_path,
        }
    }

    /// 创建测试用的配置文件
    pub async fn create_test_config(&self) -> std::io::Result<()> {
        let config_content = r#"
ip = "127.0.0.1"
port = 9999
admin_password = "$2b$12$test_hash_for_testing"

[ban]
concurrency = 2
pause_seconds = 1
prompts_dir = "./tests/fixtures/prompts"
models = ["claude-3-5-haiku-20241022"]
max_tokens = 1024
request_timeout = 5000
"#;
        tokio::fs::write(&self.config_path, config_content).await
    }

    /// 插入测试 Cookie
    pub async fn insert_test_cookie(&self, cookie: &str, status: &str) -> i64 {
        let result = sqlx::query!(
            r#"
            INSERT INTO cookies (cookie, status, created_at, updated_at)
            VALUES (?, ?, datetime('now'), datetime('now'))
            RETURNING id
            "#,
            cookie,
            status
        )
        .fetch_one(&self.db_pool)
        .await
        .expect("Failed to insert test cookie");

        result.id
    }

    /// 清空所有 Cookie
    pub async fn clear_cookies(&self) {
        sqlx::query!("DELETE FROM cookies")
            .execute(&self.db_pool)
            .await
            .expect("Failed to clear cookies");
    }

    /// 获取 Cookie 数量
    pub async fn count_cookies(&self) -> i64 {
        sqlx::query_scalar!("SELECT COUNT(*) FROM cookies")
            .fetch_one(&self.db_pool)
            .await
            .expect("Failed to count cookies")
            .unwrap_or(0)
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // 清理资源（TempDir 会自动删除）
        tracing::debug!("Cleaning up test context");
    }
}

/// 创建测试用的 prompt 文件
pub async fn create_test_prompts(dir: &std::path::Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(dir).await?;

    let prompt1 = dir.join("test_prompt1.txt");
    tokio::fs::write(&prompt1, "Test ban prompt 1").await?;

    let prompt2 = dir.join("test_prompt2.txt");
    tokio::fs::write(&prompt2, "Test ban prompt 2").await?;

    Ok(())
}

/// 生成随机测试端口
pub fn get_random_port() -> u16 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(10000..60000)
}
