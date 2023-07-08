use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fs,
    io::Read,
    ops::{Deref, DerefMut},
    path::Path,
};

pub fn get_history(wallpapers_dir: &Path) -> Result<History> {
    let path = wallpapers_dir.join("history");
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if let Ok(history) = serde_json::from_str(&contents) {
        Ok(history)
    } else {
        Ok(History::new())
    }
}
pub fn save_history(wallpapers_dir: &Path, history: &History) -> Result<()> {
    if let Ok(json) = serde_json::to_string(&history) {
        fs::write(wallpapers_dir.join("history").as_path(), json.as_bytes())?
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    arr: VecDeque<String>,
    pub idx: usize,
}
impl History {
    pub fn new() -> Self {
        History {
            arr: VecDeque::new(),
            idx: 0,
        }
    }
    pub fn prev(&mut self) -> Option<String> {
        self.idx = self.idx.saturating_sub(1);
        self.get(self.idx).cloned()
    }
    pub fn next(&mut self) -> Option<String> {
        self.idx = (self.idx + 1).clamp(0, self.arr.len().saturating_sub(1));
        self.get(self.idx).cloned()
    }
    pub fn current(&self) -> Option<String> {
        self.get(self.idx).cloned()
    }
    pub fn push(&mut self, value: String) {
        if self.current().is_some_and(|v| v == value) {
            return;
        }
        if self.idx != self.arr.len().saturating_sub(1) {
            let _ = self.arr.split_off(self.idx + 1);
        }

        if !self.arr.is_empty() {
            self.idx += 1;
        }

        self.arr.push_back(value);
    }
}

impl Deref for History {
    type Target = VecDeque<String>;
    fn deref(&self) -> &VecDeque<String> {
        &self.arr
    }
}

impl DerefMut for History {
    fn deref_mut(&mut self) -> &mut VecDeque<String> {
        &mut self.arr
    }
}
