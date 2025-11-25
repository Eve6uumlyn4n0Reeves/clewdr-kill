/// API module for ban operations
mod admin;
mod ban;
mod config;
mod error;
mod rate_limiter;
mod stats;
mod stats_history;

pub use admin::{
    AdminAction, AdminActionResult, AdminActionType, ComponentHealth, HealthCheckResult,
    HealthStatus, SystemState, SystemStatus, api_execute_admin_action, api_get_system_status,
    api_health_check,
};
pub use ban::{
    CookieCheckResponse, api_auth, api_check_cookie, api_delete_cookie, api_get_cookies,
    api_post_cookie, api_version,
};
pub use config::{
    ConfigResponse, ConfigUpdateRequest, ValidationResult, api_export_config, api_get_config,
    api_get_config_template, api_import_config, api_reset_config, api_update_config,
    api_validate_config,
};
pub use error::ApiError;
pub use rate_limiter::{RateLimiter, default_rate_limiter};
pub use stats::{
    CookieMetrics, HistoricalStats, SystemStats, api_get_cookie_metrics, api_get_historical_stats,
    api_get_system_stats, api_reset_stats,
};
pub use stats_history::{StatsSample, get_samples, record_sample};
