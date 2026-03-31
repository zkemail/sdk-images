use std::{fs, path::Path};

use anyhow::Result;
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{compute_signal_length, run_command, run_command_and_return_output};
use serde::{Deserialize, Serialize};
use slog::info;
use tera::{Context, Tera};

use crate::payload::Payload;

#[derive(Serialize, Deserialize)]
pub struct ContractData {
    pub sender_domain: String,
    pub values: Vec<Field>,
    pub external_inputs: Vec<Field>,
    pub signal_size: usize,
    pub prover_eth_address_idx: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub max_length: usize,
    pub pack_size: usize,
    pub start_idx: usize,
}

/// Render the Solidity ZKEmailVerifier contract template and write it to the given path.
pub fn create_zkemail_verifier_contract_at_path(
    contract_data: &ContractData,
    output_path: &str,
) -> Result<()> {
    // Initialize Tera
    let mut tera = Tera::default();
    tera.add_template_file("./templates/ZKEmailVerifier.sol.tera", Some("Contract.sol"))?;

    let mut context = Context::new();
    context.insert("sender_domain", &contract_data.sender_domain);
    context.insert("values", &contract_data.values);
    context.insert("external_inputs", &contract_data.external_inputs);
    context.insert("signal_size", &contract_data.signal_size);
    context.insert(
        "prover_eth_address_idx",
        &contract_data.prover_eth_address_idx,
    );

    let rendered_contract = tera.render("Contract.sol", &context)?;

    // Ensure parent directory exists, then write
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(output_path, rendered_contract)?;

    Ok(())
}

/// Render both the ZKEmailVerifier contract and IGroth16Verifier interface to the given paths.
pub fn create_zkemail_verifier_and_interface_at_paths(
    contract_data: &ContractData,
    zkemail_output_path: &str,
    igroth16_output_path: &str,
) -> Result<()> {
    create_zkemail_verifier_contract_at_path(contract_data, zkemail_output_path)?;
    create_igroth16_verifier_interface_at_path(contract_data, igroth16_output_path)?;
    Ok(())
}

/// Render the Solidity IGroth16Verifier interface template and write it to the given path.
pub fn create_igroth16_verifier_interface_at_path(
    contract_data: &ContractData,
    output_path: &str,
) -> Result<()> {
    let mut tera = Tera::default();
    tera.add_template_file(
        "./templates/IGroth16Verifier.sol.tera",
        Some("IGroth16Verifier.sol"),
    )?;

    let mut context = Context::new();
    context.insert("signal_size", &contract_data.signal_size);

    let rendered = tera.render("IGroth16Verifier.sol", &context)?;

    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(output_path, rendered)?;

    Ok(())
}

/// Render the Solidity mock Groth16Verifier contract template and write it to the given path.
pub fn create_mock_groth16_verifier_at_path(
    contract_data: &ContractData,
    output_path: &str,
) -> Result<()> {
    let mut tera = Tera::default();
    tera.add_template_file(
        "./templates/MockGroth16Verifier.sol.tera",
        Some("Groth16Verifier.sol"),
    )?;

    let mut context = Context::new();
    context.insert("signal_size", &contract_data.signal_size);

    let rendered_contract = tera.render("Groth16Verifier.sol", &context)?;
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(output_path, rendered_contract)?;

    Ok(())
}

pub fn prepare_contract_data(payload: &Payload) -> ContractData {
    let mut signal_size = 1 + 1 + 2; // For pubkey, proverETHAddress and sha256 hash of header
    let mut current_idx = 1;

    let mut values = Vec::new();
    for regex in &payload.blueprint.decomposed_regexes {
        let pack_size = compute_signal_length(regex.max_match_length as usize);
        let field = Field {
            name: regex.name.clone(),
            max_length: regex.max_match_length as usize,
            pack_size,
            start_idx: current_idx,
        };
        let mut has_public_part = false;
        for part in regex.parts.iter() {
            if part.is_public == Some(true) {
                has_public_part = true;
                if !regex.is_hashed.unwrap_or(false) {
                    let part_pack_size = compute_signal_length(part.max_length() as usize);
                    signal_size += part_pack_size;
                    current_idx += part_pack_size;
                }
            }
        }
        if has_public_part && regex.is_hashed.unwrap_or(false) {
            // Hashed regexes emit a single PackedHash public output, regardless of part count.
            signal_size += 1;
            current_idx += 1;
        }
        values.push(field);
    }

    let prover_eth_address_idx = current_idx;
    current_idx += 1; // Add 1 prover ETH address

    let mut external_inputs = Vec::new();
    for input in &payload.blueprint.external_inputs {
        let pack_size = compute_signal_length(input.max_length as usize);
        let field = Field {
            name: input.name.clone(),
            max_length: input.max_length as usize,
            pack_size,
            start_idx: current_idx,
        };
        signal_size += pack_size;
        current_idx += pack_size;
        external_inputs.push(field);
    }

    ContractData {
        sender_domain: payload.blueprint.sender_domain.clone(),
        values,
        external_inputs,
        signal_size,
        prover_eth_address_idx,
    }
}

/// Generate a Groth16 verifier contract from a zkey and write it to the given output path.
/// The contract name in the Solidity source is derived from the output filename (e.g. Groth16Verifier.sol -> Groth16Verifier).
pub async fn generate_verifier_contract(
    tmp_dir: &str,
    snarkjs_path: &str,
    zkey_file_name: &str,
    output_path: &str,
) -> Result<()> {
    // Generate the verifier contract (snarkjs writes verifier.sol to tmp_dir)
    info!(LOG, "Generating verifier contract");
    run_command(
        snarkjs_path,
        &[
            "zkey",
            "export",
            "solidityverifier",
            zkey_file_name,
            "verifier.sol",
        ],
        Some(tmp_dir),
    )
    .await?;

    let verifier_path = Path::new(tmp_dir).join("verifier.sol");
    let output = Path::new(output_path);

    // Derive contract name from output filename (e.g. Groth16Verifier.sol -> Groth16Verifier)
    let contract_name = output
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Groth16Verifier");

    // Read the verifier contract
    let content = fs::read_to_string(&verifier_path)?;
    // Patch version and rename the contract
    let updated_content = content
        .replace(
            Regex::new(r"pragma solidity .*;")
                .unwrap()
                .find(&content)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not find pragma solidity declaration in verifier contract"
                    )
                })?
                .as_str(),
            &format!("pragma solidity ^{};", "0.8.30"),
        )
        .replace(
            Regex::new(r"contract .*\{")
                .unwrap()
                .find(&content)
                .ok_or_else(|| {
                    anyhow::anyhow!("Could not find contract declaration in verifier contract")
                })?
                .as_str(),
            &format!("contract {} {{", contract_name),
        );

    // Ensure parent directory exists, then write
    if let Some(parent) = output.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(output, updated_content)?;
    fs::remove_file(&verifier_path)?;

    info!(
        LOG,
        "Wrote verifier to {} (contract {})", output_path, contract_name
    );

    Ok(())
}

pub async fn deploy_verifier_contract(chain_id: u32) -> Result<String> {
    const POLKADOT_HUB_TESTNET_CHAIN_ID: u32 = 420420417;

    info!(LOG, "Installing contract dependencies");
    run_command("yarn", &["install"], Some("tmp/contracts")).await?;

    info!(LOG, "Building contracts");
    run_command("yarn", &["build"], Some("tmp/contracts")).await?;

    info!(LOG, "Deploying contracts");
    run_command_and_return_output(
        "yarn",
        &["deploy", &chain_id.to_string()],
        Some("tmp/contracts"),
    )
    .await?;

    if chain_id == POLKADOT_HUB_TESTNET_CHAIN_ID {
        info!(
            LOG,
            "Skipping contract verification for Polkadot Hub deployment"
        );
    } else {
        info!(LOG, "Verifying contracts");
        if let Err(e) = run_command(
            "yarn",
            &["verify", &format!("chain-{}", chain_id)],
            Some("tmp/contracts"),
        )
        .await
        {
            info!(
                LOG,
                "Contract verification failed: {}. Continuing without verification.", e
            );
        }
    }

    let zk_email_verifier = read_ignition_deployed_address(chain_id)?.ok_or_else(|| {
        anyhow::anyhow!(
            "ZKEmailVerifierModule#ZKEmailVerifier not found in Ignition deployed addresses for chain-{}",
            chain_id
        )
    })?;
    info!(
        LOG,
        "ZK_EMAIL_VERIFIER Contract is at: {}", zk_email_verifier
    );

    Ok(zk_email_verifier)
}

fn read_ignition_deployed_address(chain_id: u32) -> Result<Option<String>> {
    let path = format!(
        "tmp/contracts/hh-ignition/deployments/chain-{}/deployed_addresses.json",
        chain_id
    );

    if !Path::new(&path).exists() {
        return Err(anyhow::anyhow!(
            "Ignition deployed addresses file not found at {}",
            path
        ));
    }

    let content = fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let zk_email_verifier = json
        .get("ZKEmailVerifierModule#ZKEmailVerifier")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);

    Ok(zk_email_verifier)
}

#[cfg(test)]
mod tests {
    use super::prepare_contract_data;
    use crate::payload::{Payload, UploadUrls};
    use sdk_utils::proto_types::proto_blueprint::{
        Blueprint, DecomposedRegex, DecomposedRegexPart, ExternalInput,
    };

    fn payload_with(
        decomposed_regexes: Vec<DecomposedRegex>,
        external_inputs: Vec<ExternalInput>,
    ) -> Payload {
        Payload {
            blueprint: Blueprint {
                internal_version: "v2".to_string(),
                id: "test-id".to_string(),
                title: "test".to_string(),
                description: "test".to_string(),
                slug: "test/test".to_string(),
                tags: vec![],
                email_query: "from:test.com".to_string(),
                circuit_name: "TestCircuit".to_string(),
                ignore_body_hash_check: true,
                sha_precompute_selector: "".to_string(),
                email_body_max_length: 0,
                sender_domain: "x.com".to_string(),
                enable_header_masking: false,
                enable_body_masking: false,
                client_zk_framework: 1,
                server_zk_framework: 0,
                verifier_contract_chain: 84532,
                verifier_contract_address: "".to_string(),
                is_public: true,
                created_at: None,
                updated_at: None,
                external_inputs,
                decomposed_regexes,
                client_status: 1,
                server_status: 3,
                version: 1,
                github_username: "test".to_string(),
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
            private_key: "".to_string(),
            rpc_url: "".to_string(),
            chain_id: 84532,
            etherscan_api_key: "".to_string(),
            dkim_registry_address: "".to_string(),
        }
    }

    #[test]
    fn prepare_contract_data_counts_non_hashed_public_part_lengths() {
        let payload = payload_with(
            vec![DecomposedRegex {
                name: "downloadDataLink".to_string(),
                max_match_length: 128,
                location: "body".to_string(),
                is_hashed: Some(false),
                parts: vec![
                    DecomposedRegexPart {
                        is_public: Some(false),
                        regex_def: "ready for you to download ".to_string(),
                        max_length: None,
                    },
                    DecomposedRegexPart {
                        is_public: Some(true),
                        regex_def: "[^ ]*".to_string(),
                        max_length: Some(20),
                    },
                ],
            }],
            vec![ExternalInput {
                name: "address".to_string(),
                max_length: 44,
            }],
        );

        let data = prepare_contract_data(&payload);
        assert_eq!(data.signal_size, 7);
    }

    #[test]
    fn prepare_contract_data_counts_hashed_regex_once() {
        let payload = payload_with(
            vec![DecomposedRegex {
                name: "EmailSubject".to_string(),
                max_match_length: 64,
                location: "header".to_string(),
                is_hashed: Some(true),
                parts: vec![
                    DecomposedRegexPart {
                        is_public: Some(true),
                        regex_def: "subject:".to_string(),
                        max_length: Some(20),
                    },
                    DecomposedRegexPart {
                        is_public: Some(true),
                        regex_def: "Good news: your account is now Intermediate!".to_string(),
                        max_length: Some(20),
                    },
                ],
            }],
            vec![],
        );

        let data = prepare_contract_data(&payload);
        assert_eq!(data.signal_size, 5);
    }
}
