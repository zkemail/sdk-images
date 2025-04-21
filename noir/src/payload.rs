use std::env;

use anyhow::Result;
use dotenv::dotenv;
use relayer_utils::LOG;
use sdk_utils::Blueprint;
use serde::Deserialize;
use slog::info;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub blueprint: Blueprint,
    pub upload_urls: UploadUrls,
    pub database_url: String,
    pub private_key: String,
    pub rpc_url: String,
    pub chain_id: u32,
    pub etherscan_api_key: String,
    pub dkim_registry_address: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrls {
    pub circuit: String,
    pub vk: String,
}

// Function to load the payload
pub fn load_payload() -> Result<Payload> {
    dotenv().ok();

    // Decode the base64-encoded PAYLOAD environment variable
    let decoded_payload =
        base64::decode(std::env::var("PAYLOAD").expect("PAYLOAD environment variable not set"))?;

    // Convert the decoded bytes to a string
    let payload_str = String::from_utf8(decoded_payload)?;

    // Deserialize the JSON string into a Payload struct
    let payload: Payload = serde_json::from_str(&payload_str)?;

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
    if payload.dkim_registry_address != "" {
        env::set_var("DKIM_REGISTRY", &payload.dkim_registry_address);
    }

    Ok(payload)
}
