use std::{path::PathBuf, str::FromStr};

use crate::{
    cli::Range, download::download_wallpapers, process::process_wallpapers,
    rating::KonachanRatingFilter, wallpaper_history::History, wallpaper_list::get_wallpaper_list,
};
use anyhow::Result;
use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;

pub async fn download(
    download_width: &Option<Range>,
    download_height: &Option<Range>,
    screen_width: u16,
    screen_height: u16,
    tags: &Option<String>,
    rating: &KonachanRatingFilter,
    wallpapers_dir: PathBuf,
    config_dir: PathBuf,
    cache_dir: PathBuf,
    history: History,
    token: CancellationToken,
    set_wallpaper_command_override: Option<String>,
) -> Result<()> {
    let wallpaper_list = get_wallpaper_list(
        &cache_dir,
        &download_width
            .clone()
            .unwrap_or(Range::from_str(&format!("{}..", screen_width))?),
        &download_height
            .clone()
            .unwrap_or(Range::from_str(&format!("{}..", screen_height))?),
        tags,
        rating,
    )
    .await?;

    let (downloaded_wallpapers_tx, downloaded_wallpapers_rx) = mpsc::channel(10);

    tokio::task::spawn({
        let token = token.clone();
        let wallpapers_dir = wallpapers_dir.clone();

        async move {
            select! {
                biased;
                _ = token.cancelled() => {}
                _ = async move {
                        download_wallpapers(
                            wallpaper_list,
                            &wallpapers_dir,
                            downloaded_wallpapers_tx).await

                } => {}
            }
        }
    });

    process_wallpapers(
        downloaded_wallpapers_rx,
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
