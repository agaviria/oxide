//! Configuration
use serde::Deserialize;
use failure::Fail;
use std::{
    fs::read_to_string,
    io,
    path::Path,
};
use toml;

#[derive(Clone, Deserialize)]
// Global configuration structure
pub struct Cfg {
    /// Web server configuration
    pub server: ServerCfg,
    // /// Log mechanism configuration
    // pub log: LogCfg,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
// Server configuration key/values
pub struct ServerCfg {
    /// The full server URL
    pub url: String,
}

#[derive(Debug, Fail)]
pub enum CfgError {
    #[fail(display = "Unable to read configuration file: {}", _0)]
    ReadCfgFile(#[fail(cause)] io::Error),

    #[fail(display = "Invalid format for config file: {}", _0)]
    InvalidCfgFile(#[fail(cause)]toml::de::Error),
}

impl Cfg {
    /// Creates a new `Cfg` instance using the parameters found in the given
    /// TOML configuration file.  If the file cannot be found or the file is
    /// invalid, an `Error` is returned.
    pub fn load_config_file(filename: &Path) -> Result<Cfg, CfgError> {
        let path = &filename;
        let cfg_file_str = read_to_string(path)
            .map_err(CfgError::ReadCfgFile)?;
        let cfg = toml::de::from_str(&cfg_file_str)
            .map_err(CfgError::InvalidCfgFile)?;

        Ok(cfg)
    }
}

#[cfg(test)]
mod test {
    use super::Cfg;
    use std::path::Path;

    #[test]
    fn parse_cfg_with_fields() {
        let toml = Path::new("./config_test.toml");
        let cfg_result = Cfg::load_config_file(toml);
        match cfg_result {
            Ok(cfg) => {
                assert_ne!(cfg.server.url.is_empty(), true);
            }
            Err(e) => panic!("Failed configuration parse: {:?}", e)
        }
    }
}
