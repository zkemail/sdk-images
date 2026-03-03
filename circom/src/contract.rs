use std::{collections::HashMap, env, fs, path::Path};

use anyhow::Result;
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{run_command, run_command_and_return_output};
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

    std::fs::write(output_path, rendered_contract)?;

    Ok(())
}

pub fn prepare_contract_data(payload: &Payload) -> ContractData {
    let mut signal_size = 1 + 1 + 2; // For pubkey, proverETHAddress and sha256 hash of header
    let mut current_idx = 1;

    let mut values = Vec::new();
    for regex in &payload.blueprint.decomposed_regexes {
        let pack_size = ((regex.max_match_length as f64) / 31.0).ceil() as usize;
        let field = Field {
            name: regex.name.clone(),
            max_length: regex.max_match_length as usize,
            pack_size,
            start_idx: current_idx,
        };
        for part in regex.parts.iter() {
            if part.is_public == Some(true) {
                if regex.is_hashed.unwrap_or(false) {
                    signal_size += 1;
                    current_idx += 1;
                } else {
                    signal_size += pack_size;
                    current_idx += pack_size;
                }
            }
        }
        values.push(field);
    }

    let prover_eth_address_idx = current_idx;
    current_idx += 1; // Add 1 prover ETH address

    let mut external_inputs = Vec::new();
    for input in &payload.blueprint.external_inputs {
        let pack_size = ((input.max_length as f64) / 31.0).ceil() as usize;
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
            &format!("pragma solidity ^{};", "0.8.34"),
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

pub async fn deploy_verifier_contract(payload: Payload) -> Result<String> {
    info!(LOG, "Building contracts");
    run_command("yarn", &["build"], None).await?;

    info!(LOG, "Deploying contracts");
    let output = run_command_and_return_output("yarn", &["deploy"], None).await?;

    // Parse the output to extract addresses
    let re = Regex::new(r"(DKIM_REGISTRY|GROTH16_VERIFIER|ZK_EMAIL_VERIFIER): (0x[a-fA-F0-9]{40})")
        .unwrap();
    let mut contract_addresses = HashMap::new();
    for cap in re.captures_iter(&output) {
        let contract_name = &cap[1];
        let address = &cap[2];
        contract_addresses.insert(contract_name.to_string(), address.to_string());
        info!(LOG, "{} Contract is at: {}", contract_name, address);
    }

    // Write constructor arguments to a file
    info!(LOG, "Writing constructor arguments to a file");
    let constructor_args = run_command_and_return_output(
        "cast",
        &[
            "abi-encode",
            "constructor(address,address)",
            contract_addresses.get("DKIM_REGISTRY").unwrap(),
            contract_addresses.get("GROTH16_VERIFIER").unwrap(),
        ],
        None,
    )
    .await?;

    if let Ok(_) = env::var("ETHERSCAN_API_KEY") {
        info!(LOG, "Verify contracts");

        // Verify Groth16Verifier with retries
        let mut last_error = None;
        for attempt in 1..=3 {
            info!(
                LOG,
                "Attempting to verify Groth16Verifier (attempt {}/3)", attempt
            );
            match run_command(
                "forge",
                &[
                    "verify-contract",
                    "--chain-id",
                    payload.chain_id.to_string().as_str(),
                    contract_addresses.get("GROTH16_VERIFIER").unwrap(),
                    "tmp/contracts/src/Groth16Verifier.sol:Groth16Verifier",
                ],
                None,
            )
            .await
            {
                Ok(_) => {
                    info!(LOG, "Successfully verified Groth16Verifier");
                    last_error = None;
                    break;
                }
                Err(e) => {
                    info!(
                        LOG,
                        "Attempt {}/3 failed to verify Groth16Verifier: {}", attempt, e
                    );
                    last_error = Some(e);
                    if attempt < 3 {
                        info!(LOG, "Waiting 10 seconds before retry...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    }
                }
            }
        }
        if let Some(e) = last_error {
            return Err(anyhow::anyhow!(
                "Failed to verify Groth16Verifier after 3 attempts: {}",
                e
            ));
        }

        // Delay between contract verifications
        info!(LOG, "Waiting 5 seconds before next verification...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        // Verify ZKEmailVerifier with retries
        let mut last_error = None;
        for attempt in 1..=3 {
            info!(
                LOG,
                "Attempting to verify ZKEmailVerifier (attempt {}/3)", attempt
            );
            match run_command(
                "forge",
                &[
                    "verify-contract",
                    "--chain-id",
                    payload.chain_id.to_string().as_str(),
                    "--constructor-args",
                    &constructor_args,
                    contract_addresses.get("ZK_EMAIL_VERIFIER").unwrap(),
                    "tmp/contracts/src/ZKEmailVerifier.sol:ZKEmailVerifier",
                ],
                None,
            )
            .await
            {
                Ok(_) => {
                    info!(LOG, "Successfully verified ZKEmailVerifier ");
                    last_error = None;
                    break;
                }
                Err(e) => {
                    info!(
                        LOG,
                        "Attempt {}/3 failed to verify ZKEmailVerifier: {}", attempt, e
                    );
                    last_error = Some(e);
                    if attempt < 3 {
                        info!(LOG, "Waiting 10 seconds before retry...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    }
                }
            }
        }
        if let Some(e) = last_error {
            return Err(anyhow::anyhow!(
                "Failed to verify ZKEmailVerifier after 3 attempts: {}",
                e
            ));
        }
    }

    Ok(contract_addresses
        .get("ZK_EMAIL_VERIFIER")
        .unwrap()
        .to_string())
}
