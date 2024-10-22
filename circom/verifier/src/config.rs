use std::{env, fs::File, io::Read};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainConfig {
    pub private_key: String,
    pub rpc_url: String,
    pub chain_id: u32,
    pub verification_api_key: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub bucket: String,
    pub blueprint_id: String,
    pub google_application_credentials: String,
    pub chain: ChainConfig,
    pub json_logger: bool,
}

// Function to load the configuration from a JSON file
pub fn load_config() -> Result<Config> {
    // Open the configuration file
    let mut file = File::open("config.json")
        .map_err(|e| anyhow::anyhow!("Failed to open config file: {}", e))?;

    // Read the file's content into a string
    let mut data = String::new();
    file.read_to_string(&mut data)
        .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;

    // Deserialize the JSON content into a Config struct
    let config: Config = serde_json::from_str(&data)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

    // Setting ENV
    if config.json_logger {
        env::set_var("JSON_LOGGER", "true");
    }
    if config.google_application_credentials != "" {
        env::set_var(
            "GOOGLE_APPLICATION_CREDENTIALS",
            &config.google_application_credentials,
        );
    }

    Ok(config)
}
