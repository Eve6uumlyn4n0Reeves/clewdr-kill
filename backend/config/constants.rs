use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use arc_swap::ArcSwap;
use clap::Parser;
use url::Url;

use crate::{config::ClewdrConfig, Args};

pub const CONFIG_NAME: &str = "clewdr.toml";
pub const CLAUDE_ENDPOINT: &str = "https://api.claude.ai/";

pub static ENDPOINT_URL: LazyLock<Url> = LazyLock::new(|| {
    Url::parse(CLAUDE_ENDPOINT).unwrap_or_else(|_| {
        // Fallback to a reasonable default if parsing fails
        "https://claude.ai"
            .parse()
            .expect("Default Claude URL should be valid")
    })
});

pub static CLEWDR_CONFIG: LazyLock<ArcSwap<ClewdrConfig>> = LazyLock::new(|| {
    let config = ClewdrConfig::new();
    ArcSwap::from_pointee(config)
});

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Some(path) = Args::try_parse().ok().and_then(|a| a.config) {
        path
    } else {
        #[cfg(feature = "portable")]
        {
            PORTABLE_DIR.join(CONFIG_NAME)
        }
        #[cfg(feature = "xdg")]
        {
            use etcetera::{choose_app_strategy, AppStrategy, AppStrategyArgs};
            let strategy = choose_app_strategy(AppStrategyArgs {
                top_level_domain: "org".to_string(),
                author: "Xerxes-2".to_string(),
                app_name: "clewdr".to_string(),
            })
            .unwrap_or_else(|_| {
                // Fallback strategy if choosing fails
                AppStrategy::new(AppStrategyArgs {
                    top_level_domain: "org".to_string(),
                    author: "Xerxes-2".to_string(),
                    app_name: "clewdr".to_string(),
                })
            });
            strategy.in_config_dir(CONFIG_NAME)
        }
    }
});

#[cfg(feature = "portable")]
static PORTABLE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    use crate::IS_DEV;
    if *IS_DEV {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    } else {
        std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("./clewdr"))
            .parent()
            .unwrap_or_else(|| Path::new("./"))
            .to_path_buf()
    }
});
