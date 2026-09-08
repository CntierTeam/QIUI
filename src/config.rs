//! Persist token / base URL under ~/.config/qiui/config.toml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml: {0}")]
    Toml(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub base_url: Option<String>,
    pub token: Option<String>,
    pub uid: Option<String>,
    pub user_id: Option<String>,
    pub nickname: Option<String>,
    pub user_name: Option<String>,
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("qiui")
            .join("config.toml")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let p = Self::path();
        if !p.exists() {
            return Ok(Self::default());
        }
        let s = fs::read_to_string(&p)?;
        toml::from_str(&s).map_err(|e| ConfigError::Toml(e.to_string()))
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let p = Self::path();
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self).map_err(|e| ConfigError::Toml(e.to_string()))?;
        fs::write(p, s)?;
        Ok(())
    }
}
