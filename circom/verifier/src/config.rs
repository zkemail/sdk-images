use std::env;

use anyhow::Result;
use relayer_utils::LOG;
use serde::Deserialize;
use slog::info;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub download_url: String,
    pub blueprint_id: String,
    pub database_url: String,
    pub private_key: String,
    pub rpc_url: String,
    pub chain_id: u32,
    pub etherscan_api_key: String,
    pub dkim_registry: String,
}

// Function to load the configuration from a JSON file
pub fn load_config() -> Result<Payload> {
    let payload: Payload = serde_json::from_str(
        std::env::var("PAYLOAD")
            .expect("PAYLOAD environment variable not set")
            .as_str(),
    )?;

    // Setting ENV
    info!(LOG, "Setting ENV variables");
    env::set_var("JSON_LOGGER", "true");

    if payload.private_key != "" {
        env::set_var("PRIVATE_KEY", &payload.private_key);
    }
    if payload.rpc_url != "" {
        env::set_var("RPC_URL", &payload.rpc_url);
    }
    if payload.chain_id != 0 {
        env::set_var("CHAIN_ID", &payload.chain_id.to_string());
    }
    if payload.etherscan_api_key != "" {
        env::set_var("ETHERSCAN_API_KEY", &payload.etherscan_api_key);
    }
    if payload.dkim_registry != "" {
        env::set_var("DKIM_REGISTRY", &payload.dkim_registry);
    }

    Ok(payload)
}
