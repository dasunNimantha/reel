use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub tmdb_api_key: Option<String>,
    pub last_input_directory: Option<PathBuf>,
    pub last_output_directory: Option<PathBuf>,
    pub selected_pattern: Option<String>,
}

impl AppSettings {
    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "reel", "Reel").map(|dirs| dirs.config_dir().join("settings.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(settings) = serde_json::from_str(&contents) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create config directory: {}", e))?;
            }
            let contents = serde_json::to_string_pretty(self)
                .map_err(|e| format!("Failed to serialize settings: {}", e))?;
            fs::write(&path, contents)
                .map_err(|e| format!("Failed to write settings: {}", e))?;
        }
        Ok(())
    }

    pub fn get_api_key(&self) -> Option<String> {
        self.tmdb_api_key.clone()
    }

    pub fn set_api_key(&mut self, key: Option<String>) {
        self.tmdb_api_key = key;
    }
}

