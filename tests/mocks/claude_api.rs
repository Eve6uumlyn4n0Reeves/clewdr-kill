// Claude API Mock 对象

use std::sync::{Arc, Mutex};

/// Mock Claude API 响应
#[derive(Clone, Debug)]
pub enum MockResponse {
    Success(String),
    Banned,
    RateLimited,
    Error(String),
}

/// Claude API Mock 服务器
pub struct MockClaudeApi {
    responses: Arc<Mutex<Vec<MockResponse>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockClaudeApi {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// 设置下一次响应
    pub fn add_response(&self, response: MockResponse) {
        let mut responses = self.responses.lock().unwrap();
        responses.push(response);
    }

    /// 设置多个响应（按顺序返回）
    pub fn add_responses(&self, responses: Vec<MockResponse>) {
        let mut resp_list = self.responses.lock().unwrap();
        resp_list.extend(responses);
    }

    /// 获取下一个响应
    pub fn next_response(&self) -> Option<MockResponse> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            None
        } else {
            Some(responses.remove(0))
        }
    }

    /// 获取调用次数
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// 重置 Mock
    pub fn reset(&self) {
        let mut responses = self.responses.lock().unwrap();
        responses.clear();
        let mut count = self.call_count.lock().unwrap();
        *count = 0;
    }
}

impl Default for MockClaudeApi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_api_basic() {
        let mock = MockClaudeApi::new();

        mock.add_response(MockResponse::Success("test".to_string()));

        let resp = mock.next_response();
        assert!(matches!(resp, Some(MockResponse::Success(_))));
        assert_eq!(mock.call_count(), 1);
    }

    #[test]
    fn test_mock_api_multiple_responses() {
        let mock = MockClaudeApi::new();

        mock.add_responses(vec![
            MockResponse::Success("1".to_string()),
            MockResponse::Banned,
            MockResponse::RateLimited,
        ]);

        assert!(matches!(
            mock.next_response(),
            Some(MockResponse::Success(_))
        ));
        assert!(matches!(mock.next_response(), Some(MockResponse::Banned)));
        assert!(matches!(
            mock.next_response(),
            Some(MockResponse::RateLimited)
        ));
        assert_eq!(mock.call_count(), 3);
    }

    #[test]
    fn test_mock_api_reset() {
        let mock = MockClaudeApi::new();

        mock.add_response(MockResponse::Success("test".to_string()));
        let _ = mock.next_response();

        assert_eq!(mock.call_count(), 1);

        mock.reset();

        assert_eq!(mock.call_count(), 0);
        assert!(mock.next_response().is_none());
    }
}
