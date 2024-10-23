use std::{env, fs::File, io::Read};

use anyhow::Result;
use relayer_utils::LOG;
use serde::Deserialize;
use slog::info;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainConfig {}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub bucket: String,
    pub blueprint_id: String,
    pub google_application_credentials: String,
    pub private_key: String,
    pub rpc_url: String,
    pub chain_id: u32,
    pub etherscan_api_key: String,
    pub dkim_registry: String,
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
    info!(LOG, "Setting ENV variables");
    if config.json_logger {
        env::set_var("JSON_LOGGER", "true");
    }
    if config.google_application_credentials != "" {
        env::set_var(
            "GOOGLE_APPLICATION_CREDENTIALS",
            &config.google_application_credentials,
        );
    }
    if config.private_key != "" {
        env::set_var("PRIVATE_KEY", &config.private_key);
    }
    if config.rpc_url != "" {
        env::set_var("RPC_URL", &config.rpc_url);
    }
    if config.chain_id != 0 {
        env::set_var("CHAIN_ID", &config.chain_id.to_string());
    }
    if config.etherscan_api_key != "" {
        env::set_var("ETHERSCAN_API_KEY", &config.etherscan_api_key);
    }
    if config.dkim_registry != "" {
        env::set_var("DKIM_REGISTRY", &config.dkim_registry);
    }

    Ok(config)
}
