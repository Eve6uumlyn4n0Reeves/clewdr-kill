use clewdr::{
    config::BanConfig,
    db::Database,
    services::{ban_farm::BanFarm, ban_farm::OperationMode, ban_queue::BanQueueHandle},
};
use tempfile::TempDir;

/// 确认 reload_config 会加载新的 prompt 目录并更新并发配置
#[tokio::test]
async fn reload_config_updates_prompts_and_concurrency() {
    // 禁用 worker，避免外部请求
    std::env::set_var("CLEWDR_DISABLE_WORKERS", "1");

    // 使用临时数据库
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    std::fs::File::create(&db_path).expect("create db file");
    std::env::set_var("DATABASE_PATH", db_path.display().to_string());

    let db = Database::new().await.expect("db init");
    let queue = BanQueueHandle::start_with_db(db.clone())
        .await
        .expect("queue init");

    // 创建农场实例（使用默认配置与默认 prompt 目录）
    let farm = BanFarm::spawn(queue).await.expect("farm spawn");

    // 准备新的 prompt 目录
    let new_prompt_dir = TempDir::new().unwrap();
    let prompt_file = new_prompt_dir.path().join("p1.txt");
    tokio::fs::write(&prompt_file, "NEW_PROMPT").await.unwrap();

    // 构造新的配置：修改 prompts_dir 与 concurrency
    let mut new_config = BanConfig::default();
    new_config.prompts_dir = new_prompt_dir.path().to_string_lossy().to_string();
    new_config.concurrency = 5;

    farm.reload_config(new_config).await.expect("reload");

    // prompts 已更新
    assert_eq!(farm.prompt_count().await, 1);
    let sample = farm.sample_prompt().await.expect("prompt available");
    assert!(sample.contains("NEW_PROMPT"));

    // 并发配置已更新
    assert_eq!(farm.worker_count().await, 5);

    // 在禁用 worker 模式下，模式应保持 Paused
    assert_eq!(farm.current_mode_for_test().await, OperationMode::Paused);
}
