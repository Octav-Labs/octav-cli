use serde_json::json;

use crate::config::{self, Config};
use crate::error::OctavError;

pub fn set_key(key: &str) -> Result<serde_json::Value, OctavError> {
    let config = Config {
        api_key: key.to_string(),
    };
    config::save_config(&config)?;
    Ok(json!({
        "status": "ok",
        "message": "API key saved",
        "path": config::config_path().to_string_lossy()
    }))
}

pub fn show(cli_key: Option<&str>, env_key: Option<&str>) -> Result<serde_json::Value, OctavError> {
    // Check precedence: flag > env > config file
    if let Some(key) = cli_key {
        return Ok(json!({
            "source": "cli_flag",
            "key": config::mask_key(key)
        }));
    }

    if let Some(key) = env_key {
        return Ok(json!({
            "source": "environment",
            "key": config::mask_key(key)
        }));
    }

    match config::load_config() {
        Ok(cfg) => Ok(json!({
            "source": "config_file",
            "key": config::mask_key(&cfg.api_key),
            "path": config::config_path().to_string_lossy()
        })),
        Err(_) => Err(OctavError::Config(
            "No API key configured. Use `octav auth set-key <KEY>`, set OCTAV_API_KEY, or pass --api-key.".to_string(),
        )),
    }
}
