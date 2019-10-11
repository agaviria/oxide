//! Configuration
use std::str::FromStr;

use dotenv;
use serde::{Deserialize, Deserializer};
use log::{error, LevelFilter};

// To avoid multiple IO processing, Config struct is initialized into static string.
lazy_static! {
    pub static ref CONF: Config = Config::get_config();
}

/// Global configuration structure
#[derive(Clone, Deserialize)]
pub struct Config {
    /// Web server configuration
    pub server_url: String,
    /// femme log level filter configuration
    #[serde(default="default_log_level", deserialize_with="deserialize_log_level")]
    pub log_level_filter: LevelFilter,
    /// Persistance storage configuration
    pub database_url: String,
}

impl Config {
    /// Use envy to inject dotenv and env vars into the Config struct.
    /// function panics if an error is encountered.
    fn get_config() -> Config {
        dotenv::dotenv().ok();

        match envy::from_env::<Config>() {
            Ok(config) => config,
            Err(error) => panic!("Configuration exception error: {:#?}", error),
        }
    }
}

fn default_log_level() -> LevelFilter { LevelFilter::Debug }
fn deserialize_log_level<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
where D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    match LevelFilter::from_str(&s) {
        Ok(level) => Ok(level),
        Err(e) => {
            error!("Failed to parse log level:  {}", e);
            Ok(default_log_level())
        }
    }
}

#[cfg(test)]
mod configuration_test {
    use super::Config;

    #[test]
    fn get_config() {
        let conf = get_config();
        assert_ne!(conf.database_url.is_empty(), true);
    }

    #[test]
    fn get_config_from_static_str() {
        let conf = &CONF;
        assert_ne!(conf.server_url.is_empty(), true)
    }
}
