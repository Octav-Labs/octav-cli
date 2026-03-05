use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::OctavError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
}

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".octav")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config() -> Result<Config, OctavError> {
    let path = config_path();
    let data = fs::read_to_string(&path).map_err(|_| {
        OctavError::Config(format!(
            "No config file found at {}. Use `octav auth set-key <KEY>`, set OCTAV_API_KEY, or pass --api-key.",
            path.display()
        ))
    })?;
    let config: Config = serde_json::from_str(&data).map_err(|e| {
        OctavError::Config(format!("Invalid config file at {}: {}", path.display(), e))
    })?;
    if config.api_key.is_empty() {
        return Err(OctavError::Config(
            "API key in config file is empty.".to_string(),
        ));
    }
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), OctavError> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| {
        OctavError::Config(format!(
            "Failed to create config directory {}: {}",
            dir.display(),
            e
        ))
    })?;
    let path = config_path();
    let json = serde_json::to_string_pretty(config).unwrap();
    fs::write(&path, json).map_err(|e| {
        OctavError::Config(format!(
            "Failed to write config file {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(())
}

pub fn mask_key(key: &str) -> String {
    if key.len() <= 6 {
        return "***".to_string();
    }
    let prefix = &key[..4];
    let suffix = &key[key.len() - 3..];
    format!("{}***...{}", prefix, suffix)
}
