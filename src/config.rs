use serde::Deserialize;
use std::fs;
use std::io;
use toml;

#[derive(Deserialize)]
pub struct VibraConfig {
    pub path: String,
    pub cache_size: usize,
}

impl VibraConfig {
    #[allow(unused)]
    pub fn from_file(file_path: &str) -> Result<Self, io::Error> {
        let config_content = fs::read_to_string(file_path)?;
        let config: VibraConfig = toml::from_str(&config_content)?;
        Ok(config)
    }
}