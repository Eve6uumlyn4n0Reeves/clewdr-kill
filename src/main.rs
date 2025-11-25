use clewdr::{
    self, FIG, IS_DEBUG,
    config::{CLEWDR_CONFIG, CONFIG_PATH},
    error::ClewdrError,
    version_info_colored,
};
use colored::Colorize;
use std::net::SocketAddr;
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn setup_logging() {
    let filter = if IS_DEBUG { "debug" } else { "info" };

    if let Err(e) = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%H:%M:%S%.3f".to_string(),
        ))
        .try_init()
    {
        eprintln!("Failed to initialize logging: {}", e);
    }

    #[cfg(feature = "tokio-console")]
    {
        use std::str::FromStr;
        let tokio_console_filter =
            tracing_subscriber::filter::Targets::from_str("tokio=trace,runtime=trace")
                .unwrap_or_else(|e| {
                    eprintln!("Failed to parse console filter: {}, using default", e);
                    tracing_subscriber::filter::Targets::new()
                });
        let console_layer = console_subscriber::ConsoleLayer::builder()
            .server_addr(([0, 0, 0, 0], 6669))
            .spawn();

        if let Err(e) = tracing_subscriber::registry()
            .with(console_layer.with_filter(tokio_console_filter))
            .try_init()
        {
            eprintln!("Failed to initialize console subscriber: {}", e);
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

    println!("{}\n{}", FIG, version_info_colored());

    // print info
    println!("Config dir: {}", CONFIG_PATH.display().to_string().blue());
    println!("{}", *CLEWDR_CONFIG);

    // build axum router
    // create a TCP listener
    let addr = CLEWDR_CONFIG.load().address();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let router = clewdr::router::RouterBuilder::new()
        .await
        .with_default_setup()
        .build();
    let app = router.into_make_service_with_connect_info::<SocketAddr>();
    // serve the application
    Ok(axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.unwrap_or_else(|e| {
                eprintln!("Failed to install Ctrl-C handler: {}, continuing", e);
            });
        })
        .await?)
}
