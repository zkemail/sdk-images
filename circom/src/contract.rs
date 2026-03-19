use std::{collections::HashMap, env, fs, path::Path};

use anyhow::Result;
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{run_command, run_command_and_return_output};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

    let (build_cmd, deploy_cmd, verify_cmd) = if chain_id == POLKADOT_HUB_TESTNET_CHAIN_ID {
        ("build:polka", "deploy:polka", "verify:polka")
    } else {
        ("build", "deploy", "verify")
    };

    info!(LOG, "Installing contract dependencies");
    run_command("yarn", &["install"], Some("contracts")).await?;

    info!(LOG, "Building contracts");
    run_command("yarn", &[build_cmd], Some("contracts")).await?;

    info!(LOG, "Deploying contracts");
    let output = run_command_and_return_output("yarn", &[deploy_cmd], Some("contracts")).await?;

    let mut contract_addresses = HashMap::new();
    let is_polka = chain_id == POLKADOT_HUB_TESTNET_CHAIN_ID;

    if is_polka {
        if let Ok(dkim_registry) = env::var("DKIM_REGISTRY") {
            contract_addresses.insert("DKIM_REGISTRY".to_string(), dkim_registry);
        }

        if let Some((groth16_verifier, zk_email_verifier)) =
            read_ignition_deployed_addresses(chain_id)?
        {
            contract_addresses.insert("GROTH16_VERIFIER".to_string(), groth16_verifier);
            contract_addresses.insert("ZK_EMAIL_VERIFIER".to_string(), zk_email_verifier);
        }
    }

    // Fallback parser for non-Ignition deployments (or if deployment file is not found)
    if !contract_addresses.contains_key("ZK_EMAIL_VERIFIER")
        || !contract_addresses.contains_key("GROTH16_VERIFIER")
    {
        let re = Regex::new(
            r"(DKIM_REGISTRY|GROTH16_VERIFIER|ZK_EMAIL_VERIFIER): (0x[a-fA-F0-9]{40})",
        )
        .unwrap();
        for cap in re.captures_iter(&output) {
            let contract_name = &cap[1];
            let address = &cap[2];
            contract_addresses.insert(contract_name.to_string(), address.to_string());
        }
    }

    for (contract_name, address) in &contract_addresses {
        info!(LOG, "{} Contract is at: {}", contract_name, address);
    }

    let should_verify =
        chain_id == POLKADOT_HUB_TESTNET_CHAIN_ID || env::var("ETHERSCAN_API_KEY").is_ok();

    if should_verify {
        info!(LOG, "Verifying contracts");
        if let Err(e) = run_command("yarn", &[verify_cmd], Some("contracts")).await {
            info!(
                LOG,
                "Contract verification failed: {}. Continuing without verification.", e
            );
        }
    }

    Ok(contract_addresses
        .get("ZK_EMAIL_VERIFIER")
        .ok_or_else(|| {
            anyhow::anyhow!(
                "ZK_EMAIL_VERIFIER address not found in deployment output. Raw output: {}",
                output
            )
        })?
        .to_string())
}

fn read_ignition_deployed_addresses(chain_id: u32) -> Result<Option<(String, String)>> {
    let candidate_paths = [
        format!(
            "hh-ignition/deployments/chain-{}/deployed_addresses.json",
            chain_id
        ),
        format!("ignition/deployments/chain-{}/deployed_addresses.json", chain_id),
        format!(
            "contracts/hh-ignition/deployments/chain-{}/deployed_addresses.json",
            chain_id
        ),
        format!(
            "tmp/contracts/hh-ignition/deployments/chain-{}/deployed_addresses.json",
            chain_id
        ),
    ];

    let deployed_addresses_path = candidate_paths
        .iter()
        .find(|path| Path::new(path.as_str()).exists());

    let Some(path) = deployed_addresses_path else {
        return Ok(None);
    };

    let content = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&content)?;

    let groth16_verifier = json
        .get("ZKEmailVerifierModule#Groth16Verifier")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);
    let zk_email_verifier = json
        .get("ZKEmailVerifierModule#ZKEmailVerifier")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);

    match (groth16_verifier, zk_email_verifier) {
        (Some(groth), Some(zk)) => Ok(Some((groth, zk))),
        _ => Ok(None),
    }
}
