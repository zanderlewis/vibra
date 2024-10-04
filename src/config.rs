use log::info;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use toml;

#[derive(Deserialize)]
pub struct VibraConfig {
    pub path: Option<String>,
    pub cache_size: Option<usize>,
    pub encryption_layers: Option<usize>,
}

/// Initializes the `VibraConfig` by reading the configuration from a `Vibra.toml` file.
///
/// If the `Vibra.toml` file does not exist, it uses default values for the configuration.
///
/// # Returns
///
/// * `Ok(Self)` - If the configuration is successfully initialized from the file or defaults.
/// * `Err(io::Error)` - If there is an error reading the configuration file.
///
/// # Default Values
///
/// * `path`: "vibra.db"
/// * `cache_size`: 1024
/// * `encryption_layers`: 10
///
/// # Example
///
/// ```rust
/// let config = VibraConfig::init()?;
/// ```
impl VibraConfig {
    #[allow(unused)]
    pub fn init() -> Result<Self, io::Error> {
        let file_path = "Vibra.toml";
        // Check if the file exists
        if !Path::new(file_path).exists() {
            info!("Vibra.toml not found, using default values");
            return Ok(VibraConfig {
                path: Some(String::from("vibra.db")),
                cache_size: Some(1024),
                encryption_layers: Some(10),
            });
        }

        let config_content = fs::read_to_string(file_path)?;
        let config: VibraConfig = toml::from_str(&config_content)?;

        // Fill in the default values
        let path = config.path.unwrap_or_else(|| String::from("vibra.db"));
        let cache_size = config.cache_size.unwrap_or(1024);
        let encryption_layers = config.encryption_layers.unwrap_or(10);

        Ok(VibraConfig {
            path: Some(path),
            cache_size: Some(cache_size),
            encryption_layers: Some(encryption_layers),
        })
    }
}
