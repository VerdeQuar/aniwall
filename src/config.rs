use std::{fs, io::Read, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_set_wallpaper_command")]
    pub set_wallpaper_command: String,
    #[serde(default = "default_get_screen_width_command")]
    pub get_screen_width_command: String,
    #[serde(default = "default_get_screen_height_command")]
    pub get_screen_height_command: String,
}
impl Config {
    fn new() -> Config {
        Config {
            set_wallpaper_command: default_set_wallpaper_command(),
            get_screen_width_command: default_get_screen_width_command(),
            get_screen_height_command: default_get_screen_height_command(),
        }
    }
}
fn default_get_screen_width_command() -> String {
    r#"bash -c 'hyprctl monitors | head -n 2 | grep -oP "\d+(?=x.*@)"'"#.to_owned()
}

fn default_get_screen_height_command() -> String {
    r#"bash -c 'hyprctl monitors | head -n 2 | grep -oP "(?<=x)\d+(?=@)"'"#.to_owned()
}

fn default_set_wallpaper_command() -> String {
    "swaybg --mode fill --image {}".to_owned()
}

pub fn get_config(config_dir: &Path) -> Result<Config> {
    let path = config_dir.join("config.json");
    let mut file = fs::OpenOptions::new().read(true).open(&path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if let Ok(config) = serde_json::from_str(&contents) {
        Ok(config)
    } else {
        Ok(Config::new())
    }
}
