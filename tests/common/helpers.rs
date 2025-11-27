// 测试辅助函数

use chrono::Utc;

/// 生成测试用的 Cookie 字符串
pub fn generate_test_cookie(suffix: &str) -> String {
    format!("sk-ant-sid01-test-cookie-{}-AAAA", suffix)
}

/// 生成多个测试 Cookie
pub fn generate_test_cookies(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| generate_test_cookie(&format!("{:04}", i)))
        .collect()
}

/// 验证 Cookie 格式
pub fn is_valid_cookie_format(cookie: &str) -> bool {
    cookie.starts_with("sk-ant-") && cookie.len() > 20
}

/// 生成测试用的 JWT payload
pub fn generate_test_jwt_claims() -> serde_json::Value {
    serde_json::json!({
        "sub": "admin",
        "iss": "clewdr",
        "iat": Utc::now().timestamp(),
        "exp": (Utc::now() + chrono::Duration::hours(1)).timestamp()
    })
}

/// 等待条件满足（带超时）
pub async fn wait_for_condition<F>(
    mut condition: F,
    timeout_secs: u64,
    check_interval_ms: u64,
) -> bool
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;
    }

    false
}

/// 创建测试用的配置
pub fn create_test_ban_config() -> serde_json::Value {
    serde_json::json!({
        "concurrency": 2,
        "pause_seconds": 1,
        "prompts_dir": "./tests/fixtures/prompts",
        "models": ["claude-3-5-haiku-20241022"],
        "max_tokens": 1024,
        "request_timeout": 5000
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_cookie() {
        let cookie = generate_test_cookie("test");
        assert!(cookie.starts_with("sk-ant-sid01"));
        assert!(cookie.contains("test"));
    }

    #[test]
    fn test_generate_test_cookies() {
        let cookies = generate_test_cookies(5);
        assert_eq!(cookies.len(), 5);
        assert!(cookies.iter().all(|c| c.starts_with("sk-ant-sid01")));
    }

    #[test]
    fn test_is_valid_cookie_format() {
        assert!(is_valid_cookie_format("sk-ant-sid01-test-cookie-AAAA"));
        assert!(!is_valid_cookie_format("invalid-cookie"));
        assert!(!is_valid_cookie_format("sk-ant-"));
    }
}
