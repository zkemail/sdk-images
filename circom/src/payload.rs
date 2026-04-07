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

    for (key, value) in hardhat_deploy_env_from_payload(&payload) {
        env::set_var(key, value);
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

/// Environment variables derived from [`Payload`] for Hardhat / Ignition deploy (`yarn deploy`).
/// Empty strings and `chain_id == 0` are skipped (same rules as the previous inline logic).
pub fn hardhat_deploy_env_from_payload(payload: &Payload) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if !payload.private_key.is_empty() {
        out.push(("PRIVATE_KEY".to_string(), payload.private_key.clone()));
    }
    if !payload.rpc_url.is_empty() {
        out.push(("RPC_URL".to_string(), payload.rpc_url.clone()));
    }
    if payload.chain_id != 0 {
        out.push(("CHAIN_ID".to_string(), payload.chain_id.to_string()));
    }
    if !payload.etherscan_api_key.is_empty() {
        out.push((
            "ETHERSCAN_API_KEY".to_string(),
            payload.etherscan_api_key.clone(),
        ));
    }
    if !payload.dkim_registry_address.is_empty() {
        out.push((
            "DKIM_REGISTRY".to_string(),
            payload.dkim_registry_address.clone(),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{hardhat_deploy_env_from_payload, Payload, UploadUrls};
    use sdk_utils::proto_types::proto_blueprint::Blueprint;

    fn minimal_payload(
        private_key: &str,
        rpc_url: &str,
        chain_id: u32,
        etherscan: &str,
        dkim: &str,
    ) -> Payload {
        Payload {
            blueprint: Blueprint {
                internal_version: "v2".to_string(),
                id: "00000000-0000-0000-0000-000000000001".to_string(),
                title: "t".to_string(),
                description: "t".to_string(),
                slug: "t/t".to_string(),
                tags: vec![],
                email_query: "from:x.com".to_string(),
                circuit_name: "C".to_string(),
                ignore_body_hash_check: true,
                sha_precompute_selector: "".to_string(),
                email_body_max_length: 0,
                sender_domain: "x.com".to_string(),
                enable_header_masking: false,
                enable_body_masking: false,
                client_zk_framework: 1,
                server_zk_framework: 0,
                verifier_contract_chain: chain_id as i32,
                verifier_contract_address: "".to_string(),
                is_public: true,
                created_at: None,
                updated_at: None,
                external_inputs: vec![],
                decomposed_regexes: vec![],
                client_status: 1,
                server_status: 3,
                version: 1,
                github_username: "t".to_string(),
                email_header_max_length: 1024,
                remove_soft_linebreaks: false,
                stars: 0,
                ptau: 0,
                num_local_proofs: 0,
            },
            upload_urls: UploadUrls {
                circuit: "".to_string(),
                circuit_cpp: "".to_string(),
                circuit_wasm: "".to_string(),
                witness_calculator: "".to_string(),
                generate_witness: "".to_string(),
                circuit_full_zkey: "".to_string(),
                vk: "".to_string(),
                circuit_zkey: "".to_string(),
                zkey_b: "".to_string(),
                zkey_c: "".to_string(),
                zkey_d: "".to_string(),
                zkey_e: "".to_string(),
                zkey_f: "".to_string(),
                zkey_g: "".to_string(),
                zkey_h: "".to_string(),
                zkey_i: "".to_string(),
                zkey_j: "".to_string(),
                zkey_k: "".to_string(),
                circom_regex_graphs: "".to_string(),
            },
            database_url: "".to_string(),
            private_key: private_key.to_string(),
            rpc_url: rpc_url.to_string(),
            chain_id,
            etherscan_api_key: etherscan.to_string(),
            dkim_registry_address: dkim.to_string(),
        }
    }

    #[test]
    fn hardhat_env_from_payload_maps_deploy_fields() {
        let p = minimal_payload(
            "0xabc",
            "https://rpc.example",
            420420417,
            "etherscan_key",
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
        );
        let pairs: Vec<(String, String)> = hardhat_deploy_env_from_payload(&p)
            .into_iter()
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("PRIVATE_KEY".to_string(), "0xabc".to_string()),
                ("RPC_URL".to_string(), "https://rpc.example".to_string()),
                ("CHAIN_ID".to_string(), "420420417".to_string()),
                ("ETHERSCAN_API_KEY".to_string(), "etherscan_key".to_string()),
                (
                    "DKIM_REGISTRY".to_string(),
                    "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string()
                ),
            ]
        );
    }

    #[test]
    fn hardhat_env_from_payload_skips_empty_fields() {
        let p = minimal_payload("", "", 0, "", "");
        assert!(hardhat_deploy_env_from_payload(&p).is_empty());
    }
}

