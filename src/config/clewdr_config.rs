use std::{
    fmt::{Debug, Display},
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use axum::http::uri::{Authority, Scheme};
use colored::Colorize;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use http::Uri;
use passwords::PasswordGenerator;
use serde::{Deserialize, Serialize};
use wreq::{Proxy, Url};

use super::{CONFIG_PATH, ENDPOINT_URL};
use crate::error::ClewdrError;

fn generate_password() -> String {
    let pg = PasswordGenerator {
        length: 64,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true, // Enable symbols for better security
        spaces: false,
        exclude_similar_characters: true,
        strict: true,
    };
    println!("{}", "Generating secure admin password...".green());
    pg.generate_one()
        .expect("Password generator should successfully generate a password")
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BanConfig {
    #[serde(default = "default_ban_concurrency")]
    pub concurrency: usize,
    #[serde(default = "default_ban_pause_seconds")]
    pub pause_seconds: u64,
    #[serde(default = "default_ban_prompts_dir")]
    pub prompts_dir: String,
    #[serde(default = "default_ban_models")]
    pub models: Vec<String>,
    #[serde(default = "default_ban_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_ban_request_timeout")]
    pub request_timeout: u64,
    #[serde(default = "default_ban_retry_attempts")]
    pub retry_attempts: u32,
    #[serde(default = "default_adaptive_throttling")]
    pub adaptive_throttling: bool,
    #[serde(default = "default_smart_error_handling")]
    pub smart_error_handling: bool,
    #[serde(default = "default_proxy_rotation")]
    pub proxy_rotation: bool,
    #[serde(default = "default_user_agent_rotation")]
    pub user_agent_rotation: bool,
    #[serde(default = "default_request_jitter_min")]
    pub request_jitter_min: u64,
    #[serde(default = "default_request_jitter_max")]
    pub request_jitter_max: u64,
    #[serde(default)]
    pub working_hours: WorkingHoursConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkingHoursConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_work_start")]
    pub start: String,
    #[serde(default = "default_work_end")]
    pub end: String,
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

impl Default for BanConfig {
    fn default() -> Self {
        Self {
            concurrency: default_ban_concurrency(),
            pause_seconds: default_ban_pause_seconds(),
            prompts_dir: default_ban_prompts_dir(),
            models: default_ban_models(),
            max_tokens: default_ban_max_tokens(),
            request_timeout: default_ban_request_timeout(),
            retry_attempts: default_ban_retry_attempts(),
            adaptive_throttling: default_adaptive_throttling(),
            smart_error_handling: default_smart_error_handling(),
            proxy_rotation: default_proxy_rotation(),
            user_agent_rotation: default_user_agent_rotation(),
            request_jitter_min: default_request_jitter_min(),
            request_jitter_max: default_request_jitter_max(),
            working_hours: WorkingHoursConfig::default(),
        }
    }
}

impl Default for WorkingHoursConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            start: default_work_start(),
            end: default_work_end(),
            timezone: default_timezone(),
        }
    }
}

impl BanConfig {
    pub fn max_tokens(&self) -> u32 {
        self.max_tokens
    }

    pub fn is_within_working_hours(&self) -> bool {
        if !self.working_hours.enabled {
            return true;
        }

        use chrono::{Local, NaiveTime};

        let local_time = Local::now();

        if let (Ok(start_time), Ok(end_time)) = (
            NaiveTime::parse_from_str(&self.working_hours.start, "%H:%M"),
            NaiveTime::parse_from_str(&self.working_hours.end, "%H:%M"),
        ) {
            let current_time = local_time.time();
            return current_time >= start_time && current_time <= end_time;
        }

        true // Fallback to allow if parsing fails
    }

    pub fn get_jitter_delay(&self) -> std::time::Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let delay_ms = rng.gen_range(self.request_jitter_min..=self.request_jitter_max);
        std::time::Duration::from_millis(delay_ms)
    }
}

fn default_ban_concurrency() -> usize {
    50
}

fn default_ban_pause_seconds() -> u64 {
    300 // 5 minutes
}

fn default_ban_prompts_dir() -> String {
    "./ban_prompts".to_string()
}

fn default_ban_models() -> Vec<String> {
    vec![
        "claude-3-7-sonnet-20250219".to_string(),
        "claude-sonnet-4-20250514".to_string(),
    ]
}

fn default_ban_max_tokens() -> u32 {
    512
}

fn default_ban_request_timeout() -> u64 {
    30000 // 30 seconds
}

fn default_ban_retry_attempts() -> u32 {
    3
}

fn default_adaptive_throttling() -> bool {
    true
}

fn default_smart_error_handling() -> bool {
    true
}

fn default_proxy_rotation() -> bool {
    false
}

fn default_user_agent_rotation() -> bool {
    false
}

fn default_request_jitter_min() -> u64 {
    200
}

fn default_request_jitter_max() -> u64 {
    1000
}

fn default_work_start() -> String {
    "09:00".to_string()
}

fn default_work_end() -> String {
    "18:00".to_string()
}

fn default_timezone() -> String {
    "UTC".to_string()
}

fn default_ip() -> IpAddr {
    Ipv4Addr::new(127, 0, 0, 1).into()
}

fn default_port() -> u16 {
    8484
}

/// Simplified configuration for ban operations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClewdrConfig {
    // Server settings
    #[serde(default = "default_ip")]
    ip: IpAddr,
    #[serde(default = "default_port")]
    port: u16,

    // Authentication
    #[serde(default)]
    admin_password: String,

    // Network settings
    #[serde(default)]
    pub proxy: Option<String>,

    // Ban configuration
    #[serde(default)]
    pub ban: BanConfig,

    // Skip field
    #[serde(skip)]
    pub wreq_proxy: Option<Proxy>,
}

impl Default for ClewdrConfig {
    fn default() -> Self {
        Self {
            ip: default_ip(),
            port: default_port(),
            admin_password: String::new(),
            proxy: None,
            ban: BanConfig::default(),
            wreq_proxy: None,
        }
    }
}

impl Display for ClewdrConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let authority = self.address();
        let authority: Authority = authority.to_string().parse().map_err(|_| std::fmt::Error)?;
        let web_url = Uri::builder()
            .scheme(Scheme::HTTP)
            .authority(authority.to_string())
            .path_and_query("")
            .build()
            .map_err(|_| std::fmt::Error)?;
        write!(
            f,
            "Web Admin Endpoint: {}\n\
            Web Admin Password: {}\n\
            Ban Workers: {}\n\
            Prompts Directory: {}\n\
            Models: {}",
            web_url.to_string().green().underline(),
            self.admin_password.yellow(),
            self.ban.concurrency.to_string().cyan(),
            self.ban.prompts_dir.blue(),
            self.ban.models.join(", ").magenta(),
        )?;
        if let Some(ref proxy) = self.proxy {
            writeln!(f, "\nProxy: {}", proxy.to_string().blue())?;
        }
        Ok(())
    }
}

impl ClewdrConfig {
    pub fn admin_auth(&self, key: &str) -> bool {
        key == self.admin_password
    }

    pub fn endpoint(&self) -> Url {
        ENDPOINT_URL.to_owned()
    }

    pub fn address(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }

    pub fn new() -> Self {
        let mut config: ClewdrConfig = Figment::from(Toml::file(CONFIG_PATH.as_path()))
            .admerge(Env::prefixed("CLEWDR_").split("__"))
            .extract_lossy()
            .unwrap_or_default();

        config = config.validate();
        config
    }

    pub async fn save(&self) -> Result<(), ClewdrError> {
        if let Some(parent) = CONFIG_PATH.parent()
            && !parent.exists()
        {
            tokio::fs::create_dir_all(parent).await?;
        }
        Ok(tokio::fs::write(CONFIG_PATH.as_path(), toml::ser::to_string_pretty(self)?).await?)
    }

    pub fn validate(mut self) -> Self {
        if self.admin_password.trim().is_empty() {
            self.admin_password = generate_password();
        }
        self.wreq_proxy = self.proxy.to_owned().and_then(|p| {
            Proxy::all(p)
                .inspect_err(|e| {
                    self.proxy = None;
                    tracing::error!("Failed to parse proxy: {}", e);
                })
                .ok()
        });
        self
    }

    // Setter methods for private fields
    pub fn set_ip(&mut self, ip: IpAddr) {
        self.ip = ip;
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn set_admin_password(&mut self, password: String) {
        self.admin_password = password;
    }
}
