use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    config::get_config,
    rating::{Category, Rating},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
pub struct KonachanWallpaper {
    pub md5: String,
    pub file_url: String,
    pub width: i32,
    pub height: i32,
    pub score: i32,
    pub rating: Rating,
}

pub type DownloadedImagePath = PathBuf;
pub type CroppedImagePath = PathBuf;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
pub struct CropData {
    pub cropped_image_path: CroppedImagePath,
    pub crop_offset_x: i32,
    pub crop_offset_y: i32,
}
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
pub enum Prefered {
    Original,
    Cropped,
}

#[derive(Debug, thiserror::Error)]
pub enum SetWallpaperError {
    #[error(transparent)]
    IOError(
        #[backtrace]
        #[from]
        std::io::Error,
    ),
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
pub struct Wallpaper {
    pub md5: String,
    pub original_url: String,
    pub original_width: i32,
    pub original_height: i32,
    pub score: i32,
    pub rating: Rating,
    pub prefered: Prefered,
    pub category: Option<Category>,
    pub downloaded_image_path: DownloadedImagePath,
    pub crop_data: Option<CropData>,
}

pub fn set_wallpaper(config_dir: &Path, path: &Path) -> Result<()> {
    let command = get_config(&config_dir)?.set_wallpaper_command;

    let (command_name, command_args) = command
        .split_once(" ")
        .expect("Invalid set wallpaper command, check your config");

    Command::new(command_name)
        .args(
            command_args
                .replace("{}", &path.to_string_lossy())
                .split(" "),
        )
        .spawn()?;
    Ok(())
}

impl Wallpaper {
    pub fn from_konachan(
        wallpaper: KonachanWallpaper,
        downloaded_image_path: DownloadedImagePath,
    ) -> Wallpaper {
        Wallpaper {
            md5: wallpaper.md5,
            original_url: wallpaper.file_url,
            original_width: wallpaper.width,
            original_height: wallpaper.height,
            score: wallpaper.score,
            rating: wallpaper.rating,
            category: None,
            prefered: Prefered::Original,
            downloaded_image_path,
            crop_data: None,
        }
    }

    pub fn from_md5(wallpapers_dir: &Path, md5: &String) -> Result<Self> {
        let path = wallpapers_dir.join(md5).with_extension("json");

        let content = fs::read_to_string(path)?;
        let wallpaper = serde_json::from_str(&content)?;
        Ok(wallpaper)
    }
    pub fn set_prefered(&self, config_dir: &Path) -> Result<Prefered> {
        match (&self.prefered, &self.crop_data) {
            (Prefered::Cropped, Some(crop_data)) => {
                set_wallpaper(config_dir, &crop_data.cropped_image_path)?;
            }
            (Prefered::Original, _) | (Prefered::Cropped, None) => {
                set_wallpaper(config_dir, &self.downloaded_image_path)?;
            }
        };
        Ok(self.prefered.to_owned())
    }
}
