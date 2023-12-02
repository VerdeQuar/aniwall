use std::path::PathBuf;

use super::GetSubcommand;
use crate::wallpaper::{self, Wallpaper};

use crate::{commands::HistorySubcommand, wallpaper_history::History};
use anyhow::Result;

pub fn get(
    subcommand: &GetSubcommand,
    wallpapers_dir: PathBuf,
    _config_dir: PathBuf,
    mut history: History,
) -> Result<()> {
    let mut wallpaper: Option<Wallpaper> = None;
    match subcommand {
        GetSubcommand::Md5 { md5 } => {
            wallpaper = Some(Wallpaper::from_md5(&wallpapers_dir, md5)?);
        }
        GetSubcommand::History(history_subcommand) => {
            match history_subcommand {
                HistorySubcommand::Previous => {
                    if let Some(md5) = history.prev() {
                        wallpaper = Some(Wallpaper::from_md5(&wallpapers_dir, &md5)?);
                    }
                }
                HistorySubcommand::Next => {
                    if let Some(md5) = history.next() {
                        wallpaper = Some(Wallpaper::from_md5(&wallpapers_dir, &md5)?);
                    }
                }
                HistorySubcommand::Current => {
                    if let Some(md5) = history.current() {
                        wallpaper = Some(Wallpaper::from_md5(&wallpapers_dir, &md5)?);
                    }
                }
            };
        }
    }
    if let Some(wallpaper) = wallpaper {
        if let Ok(json) = serde_json::to_string_pretty(&wallpaper) {
            println!("{}", json);
        }
    }
    Ok(())
}
