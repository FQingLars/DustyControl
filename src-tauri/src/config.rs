use std::path::PathBuf;

use crate::types::AlertRule;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub alerts: AlertsConfig,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    #[serde(default = "default_update_interval")]
    pub update_interval_ms: u64,
    #[serde(default = "default_theme")]
    pub theme: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AlertsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub events: Vec<AlertRule>,
}

fn default_update_interval() -> u64 {
    1000
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000,
            theme: "dark".to_string(),
        }
    }
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            events: Vec::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            alerts: AlertsConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = get_config_path();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            match toml::from_str(&content) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!("Warning: Failed to parse config: {}. Using defaults.", e);
                }
            }
        }
        Config::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = get_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}

fn get_config_path() -> PathBuf {
    let config_dir = dirs_config_dir();
    config_dir.join("DustyControl").join("config.toml")
}

fn dirs_config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir);
    }
    if let Ok(dir) = std::env::var("HOME") {
        return PathBuf::from(dir).join(".config");
    }
    PathBuf::from(".")
}
