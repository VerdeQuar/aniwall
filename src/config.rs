use std::{fs, io::Read, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub screen_width: Option<u16>,
    pub screen_height: Option<u16>,
    #[serde(default = "default_set_wallpaper_command")]
    pub set_wallpaper_command: String,
}
impl Config {
    fn new() -> Config {
        Config {
            screen_width: None,
            screen_height: None,
            set_wallpaper_command: default_set_wallpaper_command(),
        }
    }
}

fn default_set_wallpaper_command() -> String {
    "feh --bg-fill {}".to_owned()
}

pub fn get_config(config_dir: &Path) -> Result<Config> {
    let path = config_dir.join("config.json");
    let mut file = fs::OpenOptions::new().read(true).create(true).open(&path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if let Ok(config) = serde_json::from_str(&contents) {
        Ok(config)
    } else {
        Ok(Config::new())
    }
}
