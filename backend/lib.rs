use std::{path::PathBuf, sync::LazyLock};

use clap::Parser;
use colored::Colorize;

pub mod api;
pub mod auth;
pub mod claude_web_state;
pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod router;
pub mod services;
pub mod types;
pub mod utils;

pub const IS_DEBUG: bool = cfg!(debug_assertions);
pub static IS_DEV: LazyLock<bool> = LazyLock::new(|| std::env::var("CARGO_MANIFEST_DIR").is_ok());

pub static VERSION_INFO: LazyLock<String> =
    LazyLock::new(|| format!("profile: {}", if IS_DEBUG { "debug" } else { "release" },));

/// Returns version info with colors for terminal output
pub fn version_info_colored() -> String {
    format!(
        "profile: {}",
        if IS_DEBUG {
            "debug".yellow()
        } else {
            "release".green()
        },
    )
}

pub const FIG: &str = r#"
    //   ) )                                    //   ) ) 
   //        //  ___                   ___   / //___/ /  
  //        // //___) ) //  / /  / / //   ) / / ___ (    
 //        // //       //  / /  / / //   / / //   | |    
((____/ / // ((____   ((__( (__/ / ((___/ / //    | |    
            KILL EDITION
"#;

/// Claude Ban Tool
#[derive(Parser, Debug)]
#[command(version, about = "Claude Account Ban Tool", long_about = None)]
pub struct Args {
    #[arg(short, long)]
    /// Alternative config file
    pub config: Option<PathBuf>,
}
