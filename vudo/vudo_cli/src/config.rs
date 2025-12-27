//! VUDO CLI Configuration
//!
//! Manages global configuration for the VUDO CLI, including default settings,
//! user preferences, and Imaginarium credentials.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Global configuration for VUDO CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VudoConfig {
    /// Default author Ed25519 public key for new projects
    pub default_author: Option<String>,

    /// Imaginarium registry URL
    pub registry_url: String,

    /// API token for Imaginarium authentication
    pub api_token: Option<String>,

    /// Default fuel limit for running Spirits
    pub default_fuel: u64,

    /// Default memory limit in bytes
    pub default_memory: usize,
}

impl Default for VudoConfig {
    fn default() -> Self {
        Self {
            default_author: None,
            registry_url: "https://imaginarium.vudo.univrs.io".to_string(),
            api_token: None,
            default_fuel: 1_000_000,
            default_memory: 16 * 1024 * 1024, // 16 MB
        }
    }
}

impl VudoConfig {
    /// Load configuration from the default location
    ///
    /// Configuration file is located at:
    /// - Linux/macOS: `~/.config/vudo/config.toml`
    /// - Windows: `%APPDATA%\vudo\config.toml`
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

        let config: VudoConfig = toml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    /// Get the path to the configuration file
    fn config_path() -> Result<PathBuf> {
        Ok(Self::vudo_dir_static()?.join("config.toml"))
    }

    /// Get the VUDO directory path as a static method (internal use)
    fn vudo_dir_static() -> Result<PathBuf> {
        if cfg!(target_os = "windows") {
            // Windows: %APPDATA%\vudo
            let appdata =
                std::env::var("APPDATA").context("APPDATA environment variable not set")?;
            Ok(PathBuf::from(appdata).join("vudo"))
        } else {
            // Linux/macOS: ~/.config/vudo
            let home = std::env::var("HOME").context("HOME environment variable not set")?;
            Ok(PathBuf::from(home).join(".config").join("vudo"))
        }
    }

    /// Get the VUDO directory path as an instance method
    pub fn vudo_dir(&self) -> PathBuf {
        Self::vudo_dir_static().unwrap_or_else(|_| PathBuf::from(".vudo"))
    }

    /// Get the default registry URL
    pub fn default_registry(&self) -> Option<String> {
        Some(self.registry_url.clone())
    }
}
