#[cfg(test)]
mod tests {
    use clewdr::api::RateLimiter;

    #[tokio::test]
    async fn rate_limiter_respects_window() {
        let limiter = RateLimiter::new(2, 60);
        assert!(limiter.is_allowed("1.1.1.1").await);
        assert!(limiter.is_allowed("1.1.1.1").await);
        assert!(
            !limiter.is_allowed("1.1.1.1").await,
            "third request in window should be blocked"
        );
    }

    #[tokio::test]
    async fn rate_limiter_cleanup_removes_old_entries() {
        let limiter = RateLimiter::new(1, 1);
        assert!(limiter.is_allowed("2.2.2.2").await);
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        limiter.cleanup().await;
        assert!(
            limiter.is_allowed("2.2.2.2").await,
            "after window the request should pass"
        );
    }
}
