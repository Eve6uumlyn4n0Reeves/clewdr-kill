use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Cookie {
    pub id: i64,
    pub cookie: String,
    pub status: CookieStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub last_rate_limited_at: Option<DateTime<Utc>>,
    pub request_count: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "text")]
pub enum CookieStatus {
    Pending,
    Banned,
    Checking,
}

impl ToString for CookieStatus {
    fn to_string(&self) -> String {
        match self {
            CookieStatus::Pending => "pending".to_string(),
            CookieStatus::Banned => "banned".to_string(),
            CookieStatus::Checking => "checking".to_string(),
        }
    }
}

impl std::str::FromStr for CookieStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(CookieStatus::Pending),
            "banned" => Ok(CookieStatus::Banned),
            "checking" => Ok(CookieStatus::Checking),
            _ => Err(format!("Invalid cookie status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Stats {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub total_requests: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub avg_response_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Config {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCookie {
    pub cookie: String,
    pub status: CookieStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCookie {
    pub status: Option<CookieStatus>,
    pub last_used: Option<DateTime<Utc>>,
    pub request_count: Option<i64>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub last_rate_limited_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewStats {
    pub total_requests: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub avg_response_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub pending: Vec<Cookie>,
    pub processing: Vec<Cookie>,
    pub banned: Vec<Cookie>,
    pub total_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AggregatedStats {
    pub total_cookies: i64,
    pub pending_count: i64,
    pub banned_count: i64,
    pub total_requests: i64,
    pub avg_requests_per_cookie: f64,
}
