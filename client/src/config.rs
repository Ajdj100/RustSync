use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub remote_addr: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            remote_addr: None,
        }
    }
}

pub fn config_path() -> PathBuf {
    ProjectDirs::from("com", "RustSync", "RustSyncClient")
        .unwrap()
        .config_dir()
        .join("config.toml")
}

pub fn load_config() -> Option<ClientConfig> {
    let contents = fs::read_to_string(config_path()).ok()?;
    toml::from_str(&contents).ok()
}

pub fn save_config(config: &ClientConfig) -> std::io::Result<()> {
    let contents = toml::to_string_pretty(config).unwrap();
    fs::create_dir_all(config_path().parent().unwrap())?;
    fs::write(config_path(), contents)
}
