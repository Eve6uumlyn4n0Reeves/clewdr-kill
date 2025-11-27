mod admin;
/// API module for ban operations
mod auth;
mod ban;
mod config;
mod dead_letter;
mod error_report;
mod metrics;
mod openapi;
mod prompts;
mod rate_limiter;
mod response;
mod stats;
mod stats_history;

pub use admin::{
    api_execute_admin_action, api_get_system_status, api_health_check, AdminAction,
    AdminActionResult, AdminActionType, ComponentHealth, HealthCheckResult, HealthStatus,
    SystemState, SystemStatus,
};
pub use auth::{api_login, api_validate_token, LoginRequest, LoginResponse};
pub use ban::{
    api_check_cookie, api_delete_cookie, api_get_cookies, api_post_cookie,
    api_post_multiple_cookies, api_version, CookieCheckResponse,
};
pub use config::{
    api_export_config, api_get_config, api_get_config_template, api_import_config,
    api_reset_config, api_update_config, api_validate_config, ConfigResponse, ConfigUpdateRequest,
    ValidationResult,
};
pub use dead_letter::{api_clear_dead_letters, api_get_dead_letters, DeadLetterStats};
pub use error_report::api_report_frontend_errors;
pub use metrics::api_metrics;
pub use openapi::{api_openapi, OPENAPI_JSON};
pub use prompts::{
    api_delete_prompt, api_get_prompt, api_get_prompts, api_save_prompt, PromptFile,
};
pub use rate_limiter::{default_rate_limiter, RateLimiter};
pub use response::{
    success_message, success_response, ApiError as ResponseError, ApiResponse, EmptyResponse,
};
pub use stats::{
    api_get_cookie_metrics, api_get_historical_stats, api_get_system_stats, api_reset_stats,
    CookieMetrics, HistoricalStats, SystemStats,
};
pub use stats_history::{get_samples, record_sample, StatsSample};
