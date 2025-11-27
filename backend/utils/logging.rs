//! Logging utilities for structured logging
//! Provides macros and functions for consistent logging throughout the application

use colored::Colorize;
use tracing::{error, info, warn};

/// Print application banner with structured logging
pub fn print_banner(fig: &str, version_info: &str) {
    info!("Application starting");
    info!("Version: {}", version_info);

    // Print visual elements to stdout only
    println!("{}", fig.bright_blue());
    println!("{}\n", version_info.bright_green());
}

/// Print configuration information with structured logging
pub fn print_config_info(config_path: &str, config: &str) {
    info!("Config directory: {}", config_path);
    info!("Configuration loaded successfully");

    // Print to stdout with colors
    println!("Config dir: {}", config_path.blue());
    println!("{}", config);
}

/// Print database initialization success
pub fn print_database_init() {
    info!("Database initialized successfully");
    println!("{}", "Database initialized successfully".green());
}

/// Print password generation message
pub fn print_password_generation() {
    info!("Generating secure admin password");
    println!("{}", "Generating secure admin password...".green());
}

/// Log and print error with consistent formatting
pub fn log_error(context: &str, error: &dyn std::fmt::Display) {
    error!(error = %error, context);
    eprintln!("{} {}: {}", "Error:".red().bold(), context.yellow(), error);
}

/// Log and print warning with consistent formatting
pub fn log_warning(context: &str, message: &str) {
    warn!(context, message);
    eprintln!(
        "{} {}: {}",
        "Warning:".yellow().bold(),
        context.yellow(),
        message
    );
}

/// Log and print info with consistent formatting
pub fn log_info(context: &str, message: &str) {
    info!(context, message);
    if !cfg!(test) {
        println!("{} {}: {}", "Info:".blue().bold(), context.cyan(), message);
    }
}

/// Audit log for关键操作
pub fn audit_log(action: &str, actor: Option<&str>, detail: &str) {
    tracing::info!(
        audit = true,
        action = action,
        actor = actor.unwrap_or("system"),
        detail = detail,
        "audit"
    );
}

/// Macro for structured API request logging
#[macro_export]
macro_rules! log_api_request {
    ($method:expr, $path:expr, $user:expr) => {
        tracing::info!(method = $method, path = $path, user = $user, "API request");
    };
}

/// Macro for structured API response logging
#[macro_export]
macro_rules! log_api_response {
    ($method:expr, $path:expr, $status:expr, $duration:expr) => {
        tracing::info!(
            method = $method,
            path = $path,
            status = $status,
            duration_ms = $duration.as_millis(),
            "API response"
        );
    };
}

/// Macro for structured database operation logging
#[macro_export]
macro_rules! log_db_operation {
    ($operation:expr, $table:expr, $count:expr) => {
        tracing::debug!(
            operation = $operation,
            table = $table,
            affected_rows = $count,
            "Database operation"
        );
    };
}

/// Macro for structured cookie operation logging
#[macro_export]
macro_rules! log_cookie_operation {
    ($operation:expr, $cookie_hash:expr) => {
        tracing::info!(
            operation = $operation,
            cookie_hash = $cookie_hash,
            "Cookie operation"
        );
    };
}

/// Macro for structured performance metrics logging
#[macro_export]
macro_rules! log_performance_metric {
    ($metric_name:expr, $value:expr, $unit:expr) => {
        tracing::info!(
            metric = $metric_name,
            value = $value,
            unit = $unit,
            "Performance metric"
        );
    };
}
