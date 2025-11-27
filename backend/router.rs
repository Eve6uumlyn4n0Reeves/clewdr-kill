use axum::{
    http::Method,
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::{
    signal,
    time::{self, Duration},
};
use tower_http::cors::CorsLayer;

use crate::{
    api::*,
    auth::TokenManager,
    config::CLEWDR_CONFIG,
    db::Database,
    error::ClewdrError,
    middleware::{auth::admin_auth_middleware, request_logging::request_logging_middleware},
    services::{ban_farm::BanFarm, ban_queue::BanQueueHandle},
};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub ban_queue_handle: BanQueueHandle,
    pub rate_limiter: RateLimiter,
    pub login_rate_limiter: RateLimiter,
    pub ban_farm: Arc<BanFarm>,
    pub db: Database,
    pub token_manager: Arc<TokenManager>,
}

/// RouterBuilder for the application
pub struct RouterBuilder {
    app_state: AppState,
    inner: Router,
}

impl RouterBuilder {
    /// Creates a blank RouterBuilder instance
    /// Initializes the router with the provided application state
    ///
    /// # Arguments
    /// * `db` - Database instance to be used
    pub async fn new(db: Database) -> Result<Self, ClewdrError> {
        let queue_handle = BanQueueHandle::start_with_db(db.clone())
            .await
            .map_err(|e| {
                tracing::error!("Failed to start BanQueue: {}", e);
                e
            })?;

        let ban_farm = BanFarm::spawn(queue_handle.clone()).await.map_err(|e| {
            tracing::error!("Ban farm failed to start: {}", e);
            e
        })?;
        let rate_limiter = default_rate_limiter();
        let login_rate_limiter = RateLimiter::new(5, 60);
        let token_manager = Arc::new(TokenManager::from_env());
        {
            let rl = rate_limiter.clone();
            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(60));
                loop {
                    tokio::select! {
                        _ = interval.tick() => rl.cleanup().await,
                        _ = signal::ctrl_c() => {
                            tracing::info!("Rate limiter cleanup task shutting down");
                            break;
                        }
                    }
                }
            });
        }
        {
            let rl = login_rate_limiter.clone();
            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(60));
                loop {
                    tokio::select! {
                        _ = interval.tick() => rl.cleanup().await,
                        _ = signal::ctrl_c() => {
                            tracing::info!("Login rate limiter cleanup task shutting down");
                            break;
                        }
                    }
                }
            });
        }
        {
            let ban_farm_clone = ban_farm.clone();
            let queue_clone = queue_handle.clone();
            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(30));
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            if let Ok(info) = queue_clone.get_status().await {
                                let metrics = ban_farm_clone.strategy_metrics().await;
                                let avg_resp = metrics
                                    .values()
                                    .map(|m| m.average_response_time.as_millis() as u64)
                                    .filter(|v| *v > 0)
                                    .sum::<u64>();
                                let count = metrics.len() as u64;
                                let avg = if count > 0 { avg_resp / count } else { 0 };
                                let total_req_from_metrics: u64 =
                                    metrics.values().map(|m| m.total_requests).sum();
                                let succ: u64 = metrics.values().map(|m| m.successful_requests).sum();
                                let success_rate = if total_req_from_metrics > 0 {
                                    (succ as f64 / total_req_from_metrics as f64) * 100.0
                                } else if info.total_requests > 0 {
                                    let banned = info.banned.len() as u64;
                                    (info.total_requests.saturating_sub(banned) as f64
                                        / info.total_requests as f64)
                                        * 100.0
                                } else {
                                    0.0
                                };
                                crate::api::record_sample(crate::api::StatsSample {
                                    timestamp: chrono::Utc::now(),
                                    total_requests: info.total_requests.max(total_req_from_metrics),
                                    success_rate,
                                    average_response_time: avg,
                                });
                            }
                        }
                        _ = signal::ctrl_c() => {
                            tracing::info!("Stats sampling task shutting down");
                            break;
                        }
                    }
                }
            });
        }
        let app_state = AppState {
            ban_queue_handle: queue_handle,
            rate_limiter,
            login_rate_limiter,
            ban_farm,
            db,
            token_manager,
        };
        Ok(RouterBuilder {
            app_state,
            inner: Router::new(),
        })
    }

    /// Creates a new RouterBuilder instance
    /// Sets up routes for API endpoints and static file serving
    pub fn with_default_setup(self) -> Self {
        self.route_admin_endpoints()
            .setup_static_serving()
            .with_tower_trace()
            .with_request_logging()
            .with_cors()
    }

    /// Sets up routes for API endpoints
    fn route_admin_endpoints(mut self) -> Self {
        // Public routes (no authentication required)
        let public_router = Router::new()
            .route("/api/auth", get(api_validate_token))
            .route("/api/auth/login", post(api_login))
            .route("/api/version", get(api_version))
            .route("/api/health", get(api_health_check))
            .route("/api/docs/openapi.json", get(api_openapi))
            .route("/metrics", get(api_metrics))
            .with_state(self.app_state.clone());

        // Protected routes (authentication required)
        let protected_router = Router::new()
            .route("/api/cookies", get(api_get_cookies))
            .route("/api/cookie", post(api_post_cookie))
            .route("/api/cookie", delete(api_delete_cookie))
            .route("/api/cookies/batch", post(api_post_multiple_cookies))
            .route("/api/cookie/check", post(api_check_cookie))
            .route("/api/stats/system", get(api_get_system_stats))
            .route("/api/stats/cookies", get(api_get_cookie_metrics))
            .route("/api/stats/historical", post(api_get_historical_stats))
            .route("/api/stats/reset", post(api_reset_stats))
            .route("/api/config", get(api_get_config))
            .route("/api/config", post(api_update_config))
            .route("/api/config/reset", post(api_reset_config))
            .route("/api/config/validate", post(api_validate_config))
            .route("/api/config/export", get(api_export_config))
            .route("/api/config/import", post(api_import_config))
            .route("/api/config/templates", get(api_get_config_template))
            .route("/api/prompts", get(api_get_prompts))
            .route("/api/prompts/get", post(api_get_prompt))
            .route("/api/prompts/save", post(api_save_prompt))
            .route("/api/prompts/delete", post(api_delete_prompt))
            .route("/api/admin/action", post(api_execute_admin_action))
            .route("/api/admin/status", get(api_get_system_status))
            .with_state(self.app_state.clone())
            .layer(middleware::from_fn_with_state(
                self.app_state.clone(),
                admin_auth_middleware,
            ));

        self.inner = self.inner.merge(public_router).merge(protected_router);
        self
    }

    /// Sets up static file serving
    fn setup_static_serving(mut self) -> Self {
        #[cfg(feature = "embed-resource")]
        {
            use include_dir::{include_dir, Dir};
            const INCLUDE_STATIC: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");
            self.inner = self
                .inner
                .fallback_service(tower_serve_static::ServeDir::new(&INCLUDE_STATIC));
        }
        #[cfg(feature = "external-resource")]
        {
            use const_format::formatc;
            use tower_http::services::ServeDir;
            self.inner = self.inner.fallback_service(ServeDir::new(formatc!(
                "{}/static",
                env!("CARGO_MANIFEST_DIR")
            )));
        }
        self
    }

    /// Adds CORS support to the router
    fn with_cors(mut self) -> Self {
        use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
        use http::header::{HeaderName, HeaderValue};
        use tower_http::cors::Any;

        let config = CLEWDR_CONFIG.load();
        let mut cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_headers([
                AUTHORIZATION,
                CONTENT_TYPE,
                HeaderName::from_static("x-api-key"),
            ]);

        if let Some(origins) = config.allowed_origins() {
            let parsed: Vec<HeaderValue> = origins
                .into_iter()
                .filter_map(|origin| HeaderValue::from_str(&origin).ok())
                .collect();
            if parsed.is_empty() {
                cors = cors.allow_origin(Any);
            } else {
                cors = cors.allow_origin(parsed);
            }
        } else {
            cors = cors.allow_origin(Any);
        }

        self.inner = self.inner.layer(cors);
        self
    }

    fn with_tower_trace(mut self) -> Self {
        use tower_http::trace::TraceLayer;

        let layer = TraceLayer::new_for_http();

        self.inner = self.inner.layer(layer);
        self
    }

    /// Adds request logging middleware
    fn with_request_logging(mut self) -> Self {
        use axum::middleware;

        self.inner = self
            .inner
            .layer(middleware::from_fn(request_logging_middleware));
        self
    }

    /// Returns the configured router
    /// Finalizes the router configuration for use with axum
    pub fn build(self) -> Router {
        self.inner
    }
}
