use std::path::{Path, PathBuf};

use std::fs;

use super::SetSubcommand;
use crate::wallpaper::Wallpaper;
use crate::wallpaper_history::save_history;
use crate::{commands::HistorySubcommand, wallpaper::set_wallpaper, wallpaper_history::History};
use anyhow::Result;
use glob::glob;
use rand::seq::SliceRandom;

pub fn set(
    subcommand: &SetSubcommand,
    wallpapers_dir: PathBuf,
    config_dir: PathBuf,
    mut history: History,
) -> Result<()> {
    match subcommand {
        SetSubcommand::File { path } => {
            set_wallpaper(&config_dir, Path::new(path))?;
        }
        SetSubcommand::Md5 { md5 } => {
            let wallpaper = Wallpaper::from_md5(&wallpapers_dir, md5)?;
            wallpaper.set_prefered(&config_dir)?;
            history.push(wallpaper.md5);
        }
        SetSubcommand::Random { rating, category } => {
            let wallpapers: Vec<PathBuf> =
                glob(&format!("{}/*.json", wallpapers_dir.to_str().unwrap()))?
                    .filter_map(Result::ok)
                    .collect();
            loop {
                if let Some(path) = wallpapers.choose(&mut rand::thread_rng()) {
                    let content = fs::read_to_string(path)?;
                    let wallpaper: Wallpaper = serde_json::from_str(&content)?;
                    let current = history.current();

                    if (wallpaper.category == Some(category.to_owned()))
                        && (wallpaper.rating == *rating)
                        && (current.is_none() || current.is_some_and(|c| c != wallpaper.md5))
                    {
                        wallpaper.set_prefered(&config_dir)?;
                        history.push(wallpaper.md5);

                        break;
                    }
                }
            }
        }
        SetSubcommand::History(history_subcommand) => {
            match history_subcommand {
                HistorySubcommand::Previous => {
                    if let Some(md5) = history.prev() {
                        Wallpaper::from_md5(&wallpapers_dir, &md5)?.set_prefered(&config_dir)?;
                    }
                }
                HistorySubcommand::Next => {
                    if let Some(md5) = history.next() {
                        Wallpaper::from_md5(&wallpapers_dir, &md5)?.set_prefered(&config_dir)?;
                    }
                }
                HistorySubcommand::Current => {
                    if let Some(md5) = history.current() {
                        Wallpaper::from_md5(&wallpapers_dir, &md5)?.set_prefered(&config_dir)?;
                    }
                }
            };
        }
    }

    save_history(&wallpapers_dir, &history)?;
    Ok(())
}
