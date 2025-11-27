use std::{
    fmt::{Debug, Display},
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use axum::http::uri::{Authority, Scheme};
use bcrypt::{hash, verify, DEFAULT_COST};
use colored::Colorize;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use http::Uri;
use passwords::PasswordGenerator;
use serde::{Deserialize, Serialize};
use snafu::{GenerateImplicitData, Location};
use wreq::{Proxy, Url};

use super::{CONFIG_PATH, ENDPOINT_URL};
use crate::error::ClewdrError;
use crate::utils::print_password_generation;

fn is_bcrypt_hash(value: &str) -> bool {
    value.starts_with("$2a$") || value.starts_with("$2b$") || value.starts_with("$2y$")
}

fn hash_admin_password(password: &str) -> Result<String, ClewdrError> {
    hash(password, DEFAULT_COST).map_err(|e| ClewdrError::Whatever {
        message: "Failed to hash admin password".to_string(),
        source: Some(Box::new(e)),
    })
}

fn ensure_password_strength(password: &str) -> Result<(), ClewdrError> {
    let trimmed = password.trim();
    if trimmed.len() < 12 {
        return Err(ClewdrError::ConfigurationError {
            msg: "Admin password must be at least 12 characters".into(),
        });
    }

    let has_upper = trimmed.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = trimmed.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
    let has_symbol = trimmed.chars().any(|c| !c.is_ascii_alphanumeric());

    if has_upper && has_lower && has_digit && has_symbol {
        Ok(())
    } else {
        Err(ClewdrError::ConfigurationError {
            msg: "Password must contain uppercase, lowercase, digits, and symbols".into(),
        })
    }
}

fn generate_password() -> Result<String, ClewdrError> {
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
    print_password_generation();
    pg.generate_one().map_err(|e| {
        tracing::error!("Failed to generate password: {}", e);
        ClewdrError::InternalServerError {
            msg: format!("Password generation failed: {}", e),
        }
    })
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
        }
    }
}

impl BanConfig {
    pub fn max_tokens(&self) -> u32 {
        self.max_tokens
    }
}

fn default_ban_concurrency() -> usize {
    20 // Aggressive 模式 - 使用 Haiku 最大化并发
}

fn default_ban_pause_seconds() -> u64 {
    30 // Aggressive 模式 - 最小延迟
}

fn default_ban_prompts_dir() -> String {
    "./ban_prompts".to_string()
}

fn default_ban_models() -> Vec<String> {
    vec![
        "claude-3-5-haiku-20241022".to_string(),
        "claude-3-7-sonnet-20250219".to_string(),
    ]
}

fn default_ban_max_tokens() -> u32 {
    2048 // Aggressive 模式 - 最大化 prompt 效果
}

fn default_ban_request_timeout() -> u64 {
    30000
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

    #[serde(default)]
    pub allowed_origins: Option<Vec<String>>,

    #[serde(default)]
    pub disable_config_persistence: bool,

    // Skip field
    #[serde(skip)]
    pub wreq_proxy: Option<Proxy>,

    #[serde(skip)]
    password_from_env: bool,
}

impl Default for ClewdrConfig {
    fn default() -> Self {
        Self {
            ip: default_ip(),
            port: default_port(),
            admin_password: String::new(),
            proxy: None,
            ban: BanConfig::default(),
            allowed_origins: None,
            disable_config_persistence: false,
            wreq_proxy: None,
            password_from_env: false,
        }
    }
}

impl Display for ClewdrConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let authority = self.address();
        let authority: Authority = authority.to_string().parse().map_err(|_| std::fmt::Error)?;
        let masked_password = if self.admin_password.is_empty() {
            "[NOT SET]".to_string()
        } else if is_bcrypt_hash(&self.admin_password) {
            "[HASHED]".to_string()
        } else {
            format!(
                "{}***",
                &self.admin_password[..self.admin_password.len().min(4)]
            )
        };
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
            masked_password.yellow(),
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
        if self.admin_password.is_empty() {
            return false;
        }

        if is_bcrypt_hash(&self.admin_password) {
            verify(key, &self.admin_password).unwrap_or(false)
        } else {
            key == self.admin_password
        }
    }

    pub fn endpoint(&self) -> Url {
        ENDPOINT_URL.to_owned()
    }

    pub fn address(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }

    pub fn allowed_origins(&self) -> Option<Vec<String>> {
        self.allowed_origins.clone()
    }

    pub fn allow_persistence(&self) -> bool {
        !self.disable_config_persistence
    }

    pub fn new() -> Self {
        let mut config: ClewdrConfig = Figment::from(Toml::file(CONFIG_PATH.as_path()))
            .admerge(Env::prefixed("CLEWDR_").split("__"))
            .extract_lossy()
            .unwrap_or_default();
        config.password_from_env = std::env::var("CLEWDR_ADMIN_PASSWORD").is_ok();
        if let Ok(origins) = std::env::var("CLEWDR_ALLOWED_ORIGINS") {
            let parsed: Vec<String> = origins
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !parsed.is_empty() {
                config.allowed_origins = Some(parsed);
            }
        }
        if std::env::var("CLEWDR_DISABLE_CONFIG_WRITE").is_ok()
            || std::env::var("CLEWDR_DISABLE_CONFIG_PERSISTENCE").is_ok()
        {
            config.disable_config_persistence = true;
        }

        config = config.validate().unwrap_or_else(|e| {
            panic!("Failed to validate configuration: {}", e);
        });
        config
    }

    pub async fn save(&self) -> Result<(), ClewdrError> {
        if let Some(parent) = CONFIG_PATH.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        Ok(tokio::fs::write(CONFIG_PATH.as_path(), toml::ser::to_string_pretty(self)?).await?)
    }

    pub fn validate(mut self) -> Result<Self, ClewdrError> {
        if self.admin_password.trim().is_empty() {
            let generated = generate_password()?;
            tracing::info!("Generated admin password");
            if !cfg!(test) {
                println!("{} {}", "Generated admin password:".green(), generated);
            }
            self.admin_password = generated;
            self.password_from_env = false;
        }

        if !is_bcrypt_hash(&self.admin_password) {
            ensure_password_strength(&self.admin_password)?;
            let hashed = hash_admin_password(&self.admin_password)?;
            self.admin_password = hashed;
            if !self.password_from_env {
                if self.disable_config_persistence {
                    tracing::warn!(
                        "Configuration persistence disabled; admin password hash not written to {}",
                        CONFIG_PATH.display()
                    );
                } else if let Err(e) = self.persist_admin_password_sync() {
                    tracing::warn!("Failed to persist admin password hash: {}", e);
                }
            }
        }
        self.wreq_proxy = self.proxy.to_owned().and_then(|p| {
            Proxy::all(p)
                .inspect_err(|e| {
                    self.proxy = None;
                    tracing::error!("Failed to parse proxy: {}", e);
                })
                .ok()
        });
        Ok(self)
    }

    fn persist_admin_password_sync(&self) -> Result<(), ClewdrError> {
        if let Some(parent) = CONFIG_PATH.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|source| ClewdrError::IoError {
                    loc: Location::generate(),
                    source,
                })?;
            }
        }
        fs::write(CONFIG_PATH.as_path(), toml::ser::to_string_pretty(self)?).map_err(|source| {
            ClewdrError::IoError {
                loc: Location::generate(),
                source,
            }
        })?;
        Ok(())
    }

    // Setter methods for private fields
    pub fn set_ip(&mut self, ip: IpAddr) {
        self.ip = ip;
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn set_admin_password(&mut self, password: String) -> Result<(), ClewdrError> {
        if password.trim().is_empty() {
            return Err(ClewdrError::ConfigurationError {
                msg: "Admin password cannot be empty".into(),
            });
        }

        ensure_password_strength(&password)?;
        self.admin_password = hash_admin_password(&password)?;
        self.password_from_env = false;
        Ok(())
    }
}
