use clewdr::{
    db::pool_config::create_pool,
    db::Database,
    db::Queries,
    db::{CookieStatus, NewCookie},
    services::ban_queue::BanQueueHandle,
};

#[tokio::test]
async fn mark_processed_with_retry_succeeds() {
    // in-memory sqlite
    let pool = create_pool("sqlite::memory:").await.unwrap();
    // run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // construct Database directly (same module)
    let db = Database { pool: pool.clone() };
    let handle = BanQueueHandle::start_with_db(db).await.unwrap();

    // seed a cookie
    let _ = Queries::create_cookie(
        handle.pool(),
        NewCookie {
            cookie: "sk-ant-sid01-TESTCOOKIE1234567890".to_string(),
            status: CookieStatus::Pending,
        },
    )
    .await
    .unwrap();

    // should succeed within retries
    let res = handle
        .mark_processed_with_retry(
            "sk-ant-sid01-TESTCOOKIE1234567890".to_string(),
            false,
            None,
            None,
            None,
            3,
        )
        .await;

    assert!(res.is_ok());
}
