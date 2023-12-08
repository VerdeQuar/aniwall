#![feature(try_blocks)]
#![feature(exit_status_error)]
#![feature(linked_list_cursors)]
#![feature(async_closure)]
#![feature(error_generic_member_access)]

use crate::cli::Cli;
use crate::commands::download::download;
use crate::commands::get::get;
use crate::commands::review::review;
use crate::commands::set::set;
use crate::commands::Commands;
use anyhow::Result;
use config::get_config;
use config::save_config;

use crate::wallpaper_history::get_history;

use clap::Parser;
use directories::ProjectDirs;
use directories::UserDirs;

use tokio::fs;

use tokio_util::sync::CancellationToken;

mod cli;
mod commands;
mod config;
mod crop;
mod download;
mod process;
mod rating;
mod wallpaper;
mod wallpaper_history;
mod wallpaper_list;

#[tokio::main]
async fn main() -> Result<()> {
    let project_dirs = ProjectDirs::from("com", "Verdek", "aniwall");
    let args = Cli::parse();
    let cache_dir = args
        .cache_dir
        .or_else(|| project_dirs.clone().map(|pd| pd.cache_dir().to_path_buf()))
        .expect("Unclear cache directory, specify it using --cache-dir");

    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).await?;
    }

    let wallpapers_dir = args
        .wallpapers_dir
        .or_else(|| {
            UserDirs::new().and_then(|ud| {
                ud.picture_dir()
                    .map(|pd| pd.to_path_buf().join("wallpapers"))
            })
        })
        .expect("Unclear directory to store wallpapers, specify it using --wallpapers-dir");

    if !wallpapers_dir.exists() {
        fs::create_dir_all(&wallpapers_dir).await?;
    }

    let config_dir = args
        .config_dir
        .or_else(|| project_dirs.map(|pd| pd.config_dir().to_path_buf()))
        .expect("Unclear config directory, specify it using --config-dir");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).await?;
    }

    let mut config = get_config(&config_dir)?;

    let size: Option<_> = try {
        (
            args.screen_width.or(config.screen_width)?,
            args.screen_height.or(config.screen_height)?,
        )
    };

    let (width, height) = size.expect(
        "Unknown screen width or height, please add screen_width and screen_height to config",
    );

    let history = get_history(&wallpapers_dir)?;

    let token = CancellationToken::new();
    let token_cloned = token.clone();

    tokio::spawn({
        async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                eprintln!("Failed to wait for CTRL+C: {}", e);
                std::process::exit(1);
            } else {
                eprintln!("\nReceived interrupt signal. Shutting down server...");
                token.cancel();
            }
        }
    });

    match &args.command {
        Commands::Download {
            download_width,
            download_height,
            tags,
            rating,
        } => {
            let wallpapers_dir = wallpapers_dir.clone();
            download(
                download_width,
                download_height,
                width,
                height,
                tags,
                rating,
                wallpapers_dir,
                config_dir,
                cache_dir,
                history,
                token_cloned,
            )
            .await?
        }
        Commands::Set { subcommand } => set(subcommand, wallpapers_dir, config_dir, history)?,
        Commands::Get { subcommand } => get(subcommand, wallpapers_dir, config_dir, history)?,
        Commands::Review { subcommand } => {
            review(
                width,
                height,
                subcommand,
                wallpapers_dir,
                config_dir,
                history,
                token_cloned,
            )
            .await?
        }
    };
    Ok(())
}
