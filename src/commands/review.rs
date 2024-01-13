use std::path::PathBuf;

use glob::glob;

use crate::{
    process::process_wallpapers, rating::Category, wallpaper::Wallpaper, wallpaper_history::History,
};
use anyhow::Result;
use tokio::{fs, select, sync::mpsc};
use tokio_util::sync::CancellationToken;

use super::ReviewSubcommand;

pub async fn review(
    screen_width: u16,
    screen_height: u16,
    subcommand: &ReviewSubcommand,
    wallpapers_dir: PathBuf,
    config_dir: PathBuf,
    history: History,
    token: CancellationToken,
    set_wallpaper_command_override: Option<String>,
) -> Result<()> {
    let (wallpapers_to_review_tx, wallpapers_to_review_rx) = mpsc::channel(10);
    let history_cloned = history.clone();

    match subcommand {
        ReviewSubcommand::Current => {
            if let Some(md5) = history_cloned.current() {
                wallpapers_to_review_tx
                    .send(Wallpaper::from_md5(&wallpapers_dir, &md5)?)
                    .await?;
            }
            drop(wallpapers_to_review_tx);
        }
        ReviewSubcommand::Liked => {
            tokio::task::spawn({
                let token = token.clone();
                let wallpapers_dir = wallpapers_dir.clone();

                async move {
                    select! {
                        _ = token.cancelled() => {}
                        _ = async move || -> Result<()> {
                            for path in glob(&format!("{}/*.json", wallpapers_dir.to_str().unwrap()))?.filter_map(Result::ok) {
                                let content =
                                    fs::read_to_string(path)
                                        .await?;
                                let wallpaper: Wallpaper = serde_json::from_str(&content)?;
                                if wallpaper.category == Some(Category::Liked) {
                                    wallpapers_to_review_tx
                                        .send(wallpaper)
                                        .await
                                        ?;
                                }

                            }
                            Ok(())
                        }() => {}
                    }
                }
            });
        }
        ReviewSubcommand::Disliked => {
            tokio::task::spawn({
                let token = token.clone();
                let wallpapers_dir = wallpapers_dir.clone();

                async move {
                    select! {
                        _ = token.cancelled() => {}
                        _ = async move || -> Result<()> {
                            for path in glob(&format!("{}/*.json", wallpapers_dir.to_str().unwrap()))?.filter_map(Result::ok) {
                                let content =
                                    fs::read_to_string(path)
                                        .await?;
                                let wallpaper: Wallpaper = serde_json::from_str(&content)?;
                                if wallpaper.category == Some(Category::Disliked) {
                                    wallpapers_to_review_tx
                                        .send(wallpaper)
                                        .await
                                        ?;
                                }

                            }
                            Ok(())
                        }() => {}
                    }
                }
            });
        }
        ReviewSubcommand::Borked => {
            tokio::task::spawn({
                let token = token.clone();
                let wallpapers_dir = wallpapers_dir.clone();

                async move {
                    select! {
                        _ = token.cancelled() => {}
                        _ = async move || -> Result<()> {
                            for path in glob(&format!("{}/*.json", wallpapers_dir.to_str().unwrap()))?.filter_map(Result::ok) {
                                let content =
                                    fs::read_to_string(path)
                                        .await?;
                                let wallpaper: Wallpaper = serde_json::from_str(&content)?;
                                if wallpaper.category == Some(Category::Borked) {
                                    wallpapers_to_review_tx
                                        .send(wallpaper)
                                        .await
                                        ?;
                                }

                            }
                            Ok(())
                        }() => {}
                    }
                }
            });
        }
    }

    process_wallpapers(
        wallpapers_to_review_rx,
        token,
        history,
        wallpapers_dir,
        config_dir,
        screen_width,
        screen_height,
        set_wallpaper_command_override,
    )
    .await?;
    Ok(())
}
