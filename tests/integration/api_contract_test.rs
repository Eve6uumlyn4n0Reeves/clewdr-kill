use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use clewdr::{db::Database, router::RouterBuilder};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

/// 构造测试用的 Router，使用临时数据库与禁用的 worker，避免外部网络请求
async fn build_test_app(temp_dir: &TempDir) -> Router {
    // 环境准备：指向临时 DB，设置强口令，禁用持久化与 worker
    let db_path = temp_dir.path().join("test.db");
    std::env::set_var("DATABASE_PATH", db_path.display().to_string());
    std::env::set_var("CLEWDR_ADMIN_PASSWORD", "Str0ng!Passw0rd#");
    std::env::set_var("CLEWDR_DISABLE_CONFIG_PERSISTENCE", "1");
    std::env::set_var("CLEWDR_DISABLE_CONFIG_WRITE", "1");
    std::env::set_var("CLEWDR_DISABLE_WORKERS", "1");

    let db = Database::new().await.expect("db init");
    RouterBuilder::new(db)
        .await
        .expect("router build")
        .with_default_setup()
        .build()
}

/// 通用 helper：解析标准 ApiResponse
#[derive(Deserialize)]
struct ApiResp<T> {
    success: bool,
    data: Option<T>,
    error: Option<Value>,
}

async fn parse_api<T: DeserializeOwned>(resp: axum::response::Response) -> (StatusCode, ApiResp<T>) {
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    let parsed: ApiResp<T> =
        serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("json parse failed: {e}, body={}", String::from_utf8_lossy(&bytes)));
    (status, parsed)
}

#[tokio::test]
async fn auth_login_and_validate() {
    let temp_dir = TempDir::new().unwrap();
    let app = build_test_app(&temp_dir).await;

    // 登录
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"password":"Str0ng!Passw0rd#"}"#))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    #[derive(Deserialize)]
    struct LoginResponse {
        token: String,
        expires_at: String,
    }

    let (status, api): (StatusCode, ApiResp<LoginResponse>) = parse_api(resp).await;
    assert_eq!(status, StatusCode::OK);
    let token = api.data.as_ref().unwrap().token.clone();
    assert!(!token.is_empty());

    // 校验 token
    let req = Request::builder()
        .method("GET")
        .uri("/api/auth")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn cookie_flow_and_stats() {
    let temp_dir = TempDir::new().unwrap();
    let app = Arc::new(build_test_app(&temp_dir).await);

    // 登录获取 token
    let login_req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"password":"Str0ng!Passw0rd#"}"#))
        .unwrap();
    let resp = app.clone().oneshot(login_req).await.expect("login response");
    #[derive(Deserialize)]
    struct LoginResponse {
        token: String,
        expires_at: String,
    }
    let (_, api): (StatusCode, ApiResp<LoginResponse>) = parse_api(resp).await;
    let token = api.data.unwrap().token;

    let auth_header = ("authorization", format!("Bearer {token}"));

    // 提交 Cookie
    let cookie_value = format!(
        "sk-ant-sid01-{}-{}AA",
        "A".repeat(86),
        "B".repeat(6)
    );
    let submit_req = Request::builder()
        .method("POST")
        .uri("/api/cookie")
        .header("content-type", "application/json")
        .header(auth_header.0, auth_header.1.clone())
        .body(Body::from(format!(r#"{{"cookie":"{cookie_value}"}}"#)))
        .unwrap();
    let resp = app.clone().oneshot(submit_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 查询队列
    let list_req = Request::builder()
        .method("GET")
        .uri("/api/cookies")
        .header(auth_header.0, auth_header.1.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(list_req).await.unwrap();
    let (status, body): (StatusCode, ApiResponse<Value>) = parse_api(resp).await;
    assert_eq!(status, StatusCode::OK);
    let pending = body
        .data
        .as_ref()
        .and_then(|v| v.get("pending"))
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(pending, 1, "pending queue should contain the submitted cookie");

    // 获取系统统计
    let stats_req = Request::builder()
        .method("GET")
        .uri("/api/stats/system")
        .header(auth_header.0, auth_header.1.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(stats_req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
