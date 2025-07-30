use std::env;

use anyhow::Result;
use base64::Engine;
use dotenv::dotenv;
use relayer_utils::LOG;
use sdk_utils::proto_types::proto_blueprint::Blueprint;
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
    pub circuit_cpp: String,
    pub circuit_wasm: String,
    pub witness_calculator: String,
    pub generate_witness: String,
    pub circuit_full_zkey: String,
    pub vk: String,
    pub circuit_zkey: String,
    pub zkey_b: String,
    pub zkey_c: String,
    pub zkey_d: String,
    pub zkey_e: String,
    pub zkey_f: String,
    pub zkey_g: String,
    pub zkey_h: String,
    pub zkey_i: String,
    pub zkey_j: String,
    pub zkey_k: String,
    pub circom_regex_graphs: String,
}

// Function to load the payload
pub fn load_payload() -> Result<Payload> {
    dotenv().ok();

    // Decode the base64-encoded PAYLOAD environment variable
    let decoded_payload = base64::engine::general_purpose::STANDARD
        .decode(std::env::var("PAYLOAD").expect("PAYLOAD environment variable not set"))?;

    // Convert the decoded bytes to a string
    let payload_str = String::from_utf8(decoded_payload)?;

    // Deserialize the JSON string into a Payload struct
    let payload: Payload = serde_json::from_str(&payload_str)?;

    // Setting ENV
    info!(LOG, "Setting ENV variables");
    env::set_var("JSON_LOGGER", "true");

    if !payload.private_key.is_empty() {
        env::set_var("PRIVATE_KEY", &payload.private_key);
    }
    if !payload.rpc_url.is_empty() {
        env::set_var("RPC_URL", &payload.rpc_url);
    }
    if payload.chain_id != 0 {
        env::set_var("CHAIN_ID", payload.chain_id.to_string());
    }
    if !payload.etherscan_api_key.is_empty() {
        env::set_var("ETHERSCAN_API_KEY", &payload.etherscan_api_key);
    }
    if !payload.dkim_registry_address.is_empty() {
        env::set_var("DKIM_REGISTRY", &payload.dkim_registry_address);
    }

    // Check if TACHYON_DIR is set
    if let Ok(tachyon_dir) = std::env::var("TACHYON_DIR") {
        env::set_var("TACHYON_DIR", tachyon_dir);
    } else {
        let home_dir = std::env::var("HOME")?;
        env::set_var("TACHYON_DIR", format!("{}/tachyon", home_dir));
    }

    // If NODE_OPTIONS is not set, set it to --max-old-space-size=65536
    if std::env::var("NODE_OPTIONS").is_err() {
        env::set_var("NODE_OPTIONS", "--max-old-space-size=65536");
    }

    Ok(payload)
}
