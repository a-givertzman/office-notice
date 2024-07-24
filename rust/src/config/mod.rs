use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
///
/// Telegram bot configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BotConfig {
    connection: BotConnectionConfig,
}
impl BotConfig {
    ///
    /// Reads config from path
    pub fn read(parent: impl Into<String>, path: impl AsRef<Path>) -> Self {
        match fs::read_to_string(&path) {
            Ok(yaml_string) => {
                match serde_yaml::from_str(&yaml_string) {
                    Ok(config) => {
                        let config: BotConfig = config;
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
/// Telegram bot connection configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BotConnectionConfig {
    name: String,
    token: String,
}

