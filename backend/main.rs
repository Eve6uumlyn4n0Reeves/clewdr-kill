use clewdr::{
    self,
    config::{CLEWDR_CONFIG, CONFIG_PATH},
    db::Database,
    error::ClewdrError,
    utils::{log_error, print_banner, print_config_info, print_database_init},
    version_info_colored, FIG, IS_DEBUG,
};
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;
use std::net::SocketAddr;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn setup_logging() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        if IS_DEBUG {
            "debug".to_string()
        } else {
            "info".to_string()
        }
    });

    let fmt_layer = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%H:%M:%S%.3f".to_string(),
        ));

    let use_json = std::env::var("CLEWDR_LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    let subscriber = if use_json {
        fmt_layer.json().flatten_event(true).finish()
    } else {
        fmt_layer.finish()
    };

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        log_error("Failed to initialize logging", &e);
    }

    #[cfg(feature = "tokio-console")]
    {
        use std::str::FromStr;
        let tokio_console_filter =
            tracing_subscriber::filter::Targets::from_str("tokio=trace,runtime=trace")
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to parse console filter: {}, using default", e);
                    tracing_subscriber::filter::Targets::new()
                });
        let console_layer = console_subscriber::ConsoleLayer::builder()
            .server_addr(([0, 0, 0, 0], 6669))
            .spawn();

        if let Err(e) = tracing_subscriber::registry()
            .with(console_layer.with_filter(tokio_console_filter))
            .try_init()
        {
            tracing::error!("Failed to initialize console subscriber: {}", e);
        }
    }
}

/// Application entry point
/// Sets up logging, checks for updates, initializes the application state,
/// creates the router, and starts the server
///
/// # Returns
/// Result indicating success or failure of the application execution
#[tokio::main]
async fn main() -> Result<(), ClewdrError> {
    // Ban tool initialization
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    #[cfg(windows)]
    {
        _ = enable_ansi_support::enable_ansi_support();
    }

    // Set up simplified logging
    setup_logging();

    print_banner(FIG, &version_info_colored());

    // print info
    print_config_info(
        &CONFIG_PATH.display().to_string(),
        &CLEWDR_CONFIG.to_string(),
    );

    // Initialize database
    let db = Database::new().await?;
    print_database_init();

    // build axum router
    // create a TCP listener
    let addr = CLEWDR_CONFIG.load().address();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let router = clewdr::router::RouterBuilder::new(db)
        .await?
        .with_default_setup()
        .build();
    let app = router.into_make_service_with_connect_info::<SocketAddr>();
    // serve the application
    Ok(axum::serve(listener, app)
        .with_graceful_shutdown(async {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::warn!("Failed to install Ctrl-C handler: {}, continuing", e);
            }
            tracing::info!("Received shutdown signal");
        })
        .await?)
}
