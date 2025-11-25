/// Security and stealth management for ban operations
///
/// This module provides comprehensive security features including:
/// - Browser fingerprint randomization for anti-detection
/// - Request pattern obfuscation to mimic human behavior
/// - Timing variation and adaptive delays
/// - Proxy rotation and User-Agent cycling
/// - Session management with intelligent timeouts
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_fingerprint_randomization: bool,
    pub enable_request_pattern_obfuscation: bool,
    pub enable_timing_variation: bool,
    pub enable_proxy_rotation: bool,
    pub enable_user_agent_rotation: bool,
    pub enable_header_randomization: bool,
    pub max_requests_per_session: u32,
    pub session_timeout: Duration,
    pub stealth_level: StealthLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StealthLevel {
    Low,    // 基本隐藏
    Medium, // 中等隐藏
    High,   // 高级隐藏
    Maximum, // 极致隐藏
}

#[derive(Debug, Clone)]
pub struct FingerprintProfile {
    pub user_agent: String,
    pub screen_resolution: String,
    pub timezone: String,
    pub language: String,
    pub platform: String,
    pub webdriver: bool,
    pub plugins: Vec<String>,
    pub canvas_fingerprint: String,
    pub webgl_fingerprint: String,
}

#[derive(Debug, Clone)]
pub struct RequestPattern {
    pub interval_mean: Duration,
    pub interval_stddev: Duration,
    pub burst_probability: f64,
    pub burst_size_range: (u32, u32),
}

#[derive(Debug)]
pub struct SecurityManager {
    config: Arc<RwLock<SecurityConfig>>,
    profiles: Vec<FingerprintProfile>,
    request_patterns: Vec<RequestPattern>,
    current_session: Arc<RwLock<SessionInfo>>,
    proxy_pool: Arc<RwLock<Vec<String>>>,
}

#[derive(Debug)]
struct SessionInfo {
    start_time: Instant,
    request_count: u32,
    last_request: Option<Instant>,
    consecutive_errors: u32,
    current_profile_index: usize,
    current_proxy_index: usize,
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            request_count: 0,
            last_request: None,
            consecutive_errors: 0,
            current_profile_index: 0,
            current_proxy_index: 0,
        }
    }
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Self {
        let profiles = Self::generate_fingerprint_profiles();
        let request_patterns = Self::generate_request_patterns();

        Self {
            config: Arc::new(RwLock::new(config)),
            profiles,
            request_patterns,
            current_session: Arc::new(RwLock::new(SessionInfo::default())),
            proxy_pool: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn set_proxy_pool(&self, proxies: Vec<String>) {
        let mut pool = self.proxy_pool.write().await;
        *pool = proxies;
        info!("Proxy pool updated with {} proxies", pool.len());
    }

    pub async fn get_request_headers(&self, cookie: &str) -> HashMap<String, String> {
        let config = self.config.read().await;
        let mut headers = HashMap::new();

        if config.enable_header_randomization {
            let profile = self.get_current_profile().await;
            headers.insert("User-Agent".to_string(), profile.user_agent.clone());
            headers.insert("Accept-Language".to_string(), profile.language.clone());
            headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string());
            headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
            headers.insert("DNT".to_string(), "1".to_string());
            headers.insert("Connection".to_string(), "keep-alive".to_string());
            headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
            headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
            headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
            headers.insert("Sec-Fetch-Site".to_string(), "none".to_string());
            headers.insert("Sec-Fetch-User".to_string(), "?1".to_string());
            headers.insert("Cache-Control".to_string(), "max-age=0".to_string());
        }

        // Add custom headers for stealth
        if matches!(config.stealth_level, StealthLevel::High | StealthLevel::Maximum) {
            headers.insert("Sec-CH-UA".to_string(), "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"".to_string());
            headers.insert("Sec-CH-UA-Mobile".to_string(), "?0".to_string());
            headers.insert("Sec-CH-UA-Platform".to_string(), "\"Windows\"".to_string());
        }

        headers
    }

    pub async fn calculate_request_delay(&self) -> Duration {
        let config = self.config.read().await;
        let session = self.current_session.read().await;

        if !config.enable_timing_variation {
            return Duration::from_millis(1000);
        }

        let pattern = self.request_patterns
            .choose(&mut rand::thread_rng())
            .unwrap_or(&self.request_patterns[0]);

        // Base delay with Gaussian distribution
        let mut rng = rand::thread_rng();
        let normal_delay = pattern.interval_mean.as_millis() as f64;
        let normal_stddev = pattern.interval_stddev.as_millis() as f64;

        let delay_ms = normal_delay + rng.gen_range(-normal_stddev..normal_stddev);
        let mut delay = Duration::from_millis(delay_ms.max(100.0) as u64);

        // Add burst delay
        if rand::random::<f64>() < pattern.burst_probability {
            let burst_size = rng.gen_range(pattern.burst_size_range.0..=pattern.burst_size_range.1);
            delay *= burst_size;

            if matches!(config.stealth_level, StealthLevel::Maximum) {
                // Extra delay for maximum stealth
                delay += Duration::from_secs(rng.gen_range(5..30));
            }
        }

        // Session-based delay adjustment
        if session.request_count > config.max_requests_per_session / 2 {
            delay *= 2; // Slow down in second half of session
        }

        delay
    }

    pub async fn should_rotate_proxy(&self) -> bool {
        let config = self.config.read().await;
        if !config.enable_proxy_rotation {
            return false;
        }

        let proxy_pool = self.proxy_pool.read().await;
        if proxy_pool.len() <= 1 {
            return false;
        }

        let mut session = self.current_session.write().await;
        let rotate_probability = match config.stealth_level {
            StealthLevel::Low => 0.1,
            StealthLevel::Medium => 0.25,
            StealthLevel::High => 0.5,
            StealthLevel::Maximum => 0.75,
        };

        let mut rng = rand::thread_rng();
        if rand::random::<f64>() < rotate_probability {
            session.current_proxy_index = (session.current_proxy_index + 1) % proxy_pool.len();
            debug!("Rotating to proxy index: {}", session.current_proxy_index);
            return true;
        }

        false
    }

    pub async fn get_current_proxy(&self) -> Option<String> {
        let config = self.config.read().await;
        if !config.enable_proxy_rotation {
            return None;
        }

        let proxy_pool = self.proxy_pool.read().await;
        if proxy_pool.is_empty() {
            return None;
        }

        let session = self.current_session.read().await;
        proxy_pool.get(session.current_proxy_index).cloned()
    }

    pub async fn rotate_fingerprint(&self) -> FingerprintProfile {
        let config = self.config.read().await;
        if !config.enable_fingerprint_randomization {
            return self.profiles[0].clone();
        }

        let mut session = self.current_session.write().await;
        session.current_profile_index = (session.current_profile_index + 1) % self.profiles.len();

        let profile = &self.profiles[session.current_profile_index];
        debug!("Rotated to fingerprint profile index: {}", session.current_profile_index);
        profile.clone()
    }

    pub async fn update_session_stats(&self, success: bool) {
        let mut session = self.current_session.write().await;
        session.request_count += 1;
        session.last_request = Some(Instant::now());

        if success {
            session.consecutive_errors = 0;
        } else {
            session.consecutive_errors += 1;
        }

        // Check session timeout
        let config = self.config.read().await;
        if session.start_time.elapsed() > config.session_timeout {
            info!("Session timeout reached, resetting session");
            *session = SessionInfo::default();
        }
    }

    pub async fn is_session_limit_reached(&self) -> bool {
        let config = self.config.read().await;
        let session = self.current_session.read().await;

        session.request_count >= config.max_requests_per_session
    }

    pub async fn reset_session(&self) {
        let mut session = self.current_session.write().await;
        *session = SessionInfo::default();
        info!("Session reset");
    }

    async fn get_current_profile(&self) -> FingerprintProfile {
        let session = self.current_session.read().await;
        self.profiles.get(session.current_profile_index)
            .cloned()
            .unwrap_or_else(|| self.profiles[0].clone())
    }

    fn generate_fingerprint_profiles() -> Vec<FingerprintProfile> {
        vec![
            FingerprintProfile {
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                screen_resolution: "1920x1080".to_string(),
                timezone: "America/New_York".to_string(),
                language: "en-US,en;q=0.9".to_string(),
                platform: "Win32".to_string(),
                webdriver: false,
                plugins: vec!["Chrome PDF Plugin".to_string(), "Chrome PDF Viewer".to_string()],
                canvas_fingerprint: "canvas_hash_1".to_string(),
                webgl_fingerprint: "webgl_hash_1".to_string(),
            },
            FingerprintProfile {
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                screen_resolution: "2560x1440".to_string(),
                timezone: "America/Los_Angeles".to_string(),
                language: "en-US,en;q=0.8".to_string(),
                platform: "MacIntel".to_string(),
                webdriver: false,
                plugins: vec!["Chrome PDF Plugin".to_string(), "Chrome PDF Viewer".to_string()],
                canvas_fingerprint: "canvas_hash_2".to_string(),
                webgl_fingerprint: "webgl_hash_2".to_string(),
            },
            FingerprintProfile {
                user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                screen_resolution: "1366x768".to_string(),
                timezone: "Europe/London".to_string(),
                language: "en-GB,en;q=0.9".to_string(),
                platform: "Linux x86_64".to_string(),
                webdriver: false,
                plugins: vec!["Chrome PDF Plugin".to_string(), "Chrome PDF Viewer".to_string()],
                canvas_fingerprint: "canvas_hash_3".to_string(),
                webgl_fingerprint: "webgl_hash_3".to_string(),
            },
            FingerprintProfile {
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0".to_string(),
                screen_resolution: "1680x1050".to_string(),
                timezone: "Europe/Paris".to_string(),
                language: "fr,fr-FR;q=0.8,en-US;q=0.5,en;q=0.3".to_string(),
                platform: "Win32".to_string(),
                webdriver: false,
                plugins: vec!["PDF Viewer".to_string(), "WebEx64 General Plugin Container".to_string()],
                canvas_fingerprint: "canvas_hash_4".to_string(),
                webgl_fingerprint: "webgl_hash_4".to_string(),
            },
        ]
    }

    fn generate_request_patterns() -> Vec<RequestPattern> {
        vec![
            RequestPattern {
                interval_mean: Duration::from_millis(2000),
                interval_stddev: Duration::from_millis(500),
                burst_probability: 0.1,
                burst_size_range: (2, 5),
            },
            RequestPattern {
                interval_mean: Duration::from_millis(5000),
                interval_stddev: Duration::from_millis(1000),
                burst_probability: 0.05,
                burst_size_range: (3, 8),
            },
            RequestPattern {
                interval_mean: Duration::from_millis(10000),
                interval_stddev: Duration::from_millis(2000),
                burst_probability: 0.02,
                burst_size_range: (5, 15),
            },
        ]
    }

    pub async fn get_security_status(&self) -> SecurityStatus {
        let config = self.config.read().await;
        let session = self.current_session.read().await;
        let proxy_pool = self.proxy_pool.read().await;

        SecurityStatus {
            stealth_level: config.stealth_level.clone(),
            session_active: session.start_time.elapsed() < config.session_timeout,
            requests_in_session: session.request_count,
            session_time_remaining: config.session_timeout.saturating_sub(session.start_time.elapsed()),
            consecutive_errors: session.consecutive_errors,
            available_proxies: proxy_pool.len(),
            current_proxy_index: if proxy_pool.is_empty() { None } else { Some(session.current_proxy_index) },
            current_profile_index: session.current_profile_index,
            enabled_features: SecurityFeatures {
                fingerprint_randomization: config.enable_fingerprint_randomization,
                request_pattern_obfuscation: config.enable_request_pattern_obfuscation,
                timing_variation: config.enable_timing_variation,
                proxy_rotation: config.enable_proxy_rotation,
                user_agent_rotation: config.enable_user_agent_rotation,
                header_randomization: config.enable_header_randomization,
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct SecurityStatus {
    pub stealth_level: StealthLevel,
    pub session_active: bool,
    pub requests_in_session: u32,
    pub session_time_remaining: Duration,
    pub consecutive_errors: u32,
    pub available_proxies: usize,
    pub current_proxy_index: Option<usize>,
    pub current_profile_index: usize,
    pub enabled_features: SecurityFeatures,
}

#[derive(Debug, Serialize, Clone)]
pub struct SecurityFeatures {
    pub fingerprint_randomization: bool,
    pub request_pattern_obfuscation: bool,
    pub timing_variation: bool,
    pub proxy_rotation: bool,
    pub user_agent_rotation: bool,
    pub header_randomization: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_fingerprint_randomization: true,
            enable_request_pattern_obfuscation: true,
            enable_timing_variation: true,
            enable_proxy_rotation: false,
            enable_user_agent_rotation: true,
            enable_header_randomization: true,
            max_requests_per_session: 1000,
            session_timeout: Duration::from_secs(3600), // 1 hour
            stealth_level: StealthLevel::Medium,
        }
    }
}

impl Default for StealthLevel {
    fn default() -> Self {
        StealthLevel::Medium
    }
}