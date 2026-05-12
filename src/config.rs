use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub template_path: PathBuf,
    pub history1_path: PathBuf,
    pub history2_path: PathBuf,
    pub output_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            template_path: PathBuf::new(),
            history1_path: PathBuf::new(),
            history2_path: PathBuf::new(),
            output_path: PathBuf::new(),
        }
    }
}

impl Config {
    fn config_path() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_default()
            .join("seat_generator_config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }
}
