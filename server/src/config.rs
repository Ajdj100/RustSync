use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};
use tokio::io;
use toml;

pub enum ConfigError {
    NotFound,
    ReadError(io::Error),
    ParseError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub addr: String,
    pub backup_dir: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let default_path = ProjectDirs::from("com", "RustSync", "RustSyncServer")
            .unwrap()
            .data_dir()
            .join("backups")
            .to_string_lossy()
            .into_owned();

        Self {
            addr: "0.0.0.0:2600".to_string(),
            backup_dir: default_path,
        }
    }
}

pub fn config_path() -> PathBuf {
    ProjectDirs::from("com", "RustSync", "RustSyncServer")
        .unwrap()
        .config_dir()
        .join("config.toml")
}

pub fn load_config() -> Result<ServerConfig, ConfigError> {
    let path = config_path();

    if !path.exists() {
        return Err(ConfigError::NotFound);
    }

    let contents = fs::read_to_string(path).map_err(ConfigError::ReadError)?;

    toml::from_str(&contents).map_err(|e| ConfigError::ParseError(e.to_string()))
}

pub fn save_config(config: &ServerConfig) -> std::io::Result<()> {
    let contents = toml::to_string_pretty(config).unwrap();
    fs::create_dir_all(config_path().parent().unwrap())?;
    fs::write(config_path(), contents)
}

pub fn load_or_create_config() -> ServerConfig {
    let config = match load_config() {
        Ok(config) => config,
        Err(ConfigError::NotFound) => {
            eprintln!("No config found, creating default");
            let config = ServerConfig::default();
            save_config(&config);
            config
        }
        Err(_e) => {
            eprintln!("Failed to load config");
            std::process::exit(1);
        }
    };

    return config;
}