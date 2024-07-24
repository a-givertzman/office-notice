use std::{fs, path::Path};
use serde::{Deserialize, Serialize};
///
/// App configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub bot: BotConfig,
}
impl AppConfig {
    ///
    /// Reads config from path
    pub fn read(parent: impl Into<String>, path: impl AsRef<Path>) -> Self {
        match fs::read_to_string(&path) {
            Ok(yaml_string) => {
                log::info!("BotConfig.read | Path to config: {:?}", path.as_ref());
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        log::info!("BotConfig.read | config: {:?}", path.as_ref());
                        let config: AppConfig = config;
                        config
                    }
                    Err(err) => {
                        panic!("BotConfig.read | Error in config: {:?}\n\terror: {:?}", yaml_string, err)
                    }
                }
            }
            Err(err) => {
                panic!("BotConfig.read | File '{:?}' reading error: {:?}", path.as_ref(), err)
            }
        }
    }
}
///
/// Telegram bot configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BotConfig {
    pub connection: BotConnectionConfig,
}
///
/// Telegram bot connection configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BotConnectionConfig {
    pub name: String,
    pub token: String,
}

