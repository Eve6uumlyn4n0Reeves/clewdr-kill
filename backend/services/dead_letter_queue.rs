use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, warn};

/// 死信队列条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    pub cookie: String,
    pub operation: String, // "mark_processed", "submit", etc.
    pub error_message: String,
    pub retry_count: u32,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// 死信队列 - 记录所有写库失败的操作
/// 用于事后分析和手动恢复
#[derive(Clone, Debug)]
pub struct DeadLetterQueue {
    entries: Arc<RwLock<Vec<DeadLetterEntry>>>,
    max_size: usize,
}

impl DeadLetterQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            max_size,
        }
    }

    /// 添加失败条目到死信队列
    pub async fn push(&self, entry: DeadLetterEntry) {
        let mut entries = self.entries.write().await;

        // 记录审计日志
        error!(
            audit = true,
            action = "dead_letter_push",
            cookie = %entry.cookie,
            operation = %entry.operation,
            error = %entry.error_message,
            retry_count = entry.retry_count,
            "Dead letter: {} failed after {} retries: {}",
            entry.operation,
            entry.retry_count,
            entry.error_message
        );

        // 如果超过最大容量,移除最旧的条目
        if entries.len() >= self.max_size {
            warn!(
                "Dead letter queue full ({}/{}), removing oldest entry",
                entries.len(),
                self.max_size
            );
            entries.remove(0);
        }

        entries.push(entry);
    }

    /// 获取所有死信条目
    pub async fn get_all(&self) -> Vec<DeadLetterEntry> {
        self.entries.read().await.clone()
    }

    /// 获取死信队列大小
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// 清空死信队列
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        let count = entries.len();
        entries.clear();

        if count > 0 {
            warn!(
                audit = true,
                action = "dead_letter_clear",
                cleared_count = count,
                "Cleared {} dead letter entries",
                count
            );
        }
    }

    /// 获取指定cookie的失败历史
    pub async fn get_by_cookie(&self, cookie: &str) -> Vec<DeadLetterEntry> {
        self.entries
            .read()
            .await
            .iter()
            .filter(|e| e.cookie == cookie)
            .cloned()
            .collect()
    }
}

impl Default for DeadLetterQueue {
    fn default() -> Self {
        Self::new(1000) // 默认保留1000条失败记录
    }
}
