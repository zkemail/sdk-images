use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{run_command, run_command_and_return_output};
use serde::Serialize;
use slog::info;
use tera::{Context, Tera};

use crate::payload::Payload;

#[derive(Serialize)]
pub struct ContractData {
    pub sender_domain: String,
    pub values: Vec<Field>,
    pub external_inputs: Vec<Field>,
    pub signal_size: usize,
    pub prover_eth_address_idx: usize,
}

#[derive(Serialize)]
pub struct Field {
    pub name: String,
    pub max_length: usize,
    pub pack_size: usize,
    pub start_idx: usize,
}

pub fn create_contract(contract_data: &ContractData) -> Result<()> {
    // Initialize Tera
    let mut tera = Tera::default();
    tera.add_template_file("./templates/template.sol.tera", Some("Contract.sol"))?;

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

    let re = regex::Regex::new(r"\n+").unwrap();

    let cleaned_contract = re.replace_all(&rendered_contract, "\n").to_string();

    // Write the rendered template to a file
    std::fs::write("tmp/Contract.sol", cleaned_contract)?;

    Ok(())
}

pub fn prepare_contract_data(payload: &Payload) -> ContractData {
    let mut signal_size = 1 + 1 + 2; // For pubkey, proverETHAddress and sha256 hash of header
    let mut current_idx = 1;

    let mut values = Vec::new();
    if let Some(decomposed_regexes) = &payload.blueprint.decomposed_regexes {
        for regex in decomposed_regexes {
            let pack_size = ((regex.max_length as f64) / 31.0).ceil() as usize;
            let field = Field {
                name: regex.name.clone(),
                max_length: regex.max_length,
                pack_size,
                start_idx: current_idx,
            };
            for part in regex.parts.iter() {
                if part.is_public {
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
    }

    let prover_eth_address_idx = current_idx;
    current_idx += 1; // Add 1 prover ETH address

    let mut external_inputs = Vec::new();
    if let Some(inputs) = &payload.blueprint.external_inputs {
        for input in inputs.iter() {
            let pack_size = ((input.max_length as f64) / 31.0).ceil() as usize;
            let field = Field {
                name: input.name.clone(),
                max_length: input.max_length,
                pack_size,
                start_idx: current_idx,
            };
            signal_size += pack_size;
            current_idx += pack_size;
            external_inputs.push(field);
        }
    }

    let sender_domain = payload
        .blueprint
        .sender_domain
        .clone()
        .expect("Sender domain not found");

    ContractData {
        sender_domain,
        values,
        external_inputs,
        signal_size,
        prover_eth_address_idx,
    }
}

pub async fn generate_verifier_contract(
    tmp_dir: &str,
    snarkjs_path: &str,
    zkey_file_name: &str,
    old_contract_name: &str,
    contract_name: &str,
) -> Result<()> {
    // Generate the verifier contract
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

    // Path to the generated verifier
    let verifier_path = Path::new(tmp_dir).join("verifier.sol");
    // Path to the renamed verifier
    let renamed_path = Path::new(tmp_dir).join(format!("{}.sol", contract_name));

    // Read, patch, and rename the contract
    let updated_content = fs::read_to_string(&verifier_path)?
        .replace("pragma solidity ^0.6.11;", "pragma solidity ^0.8.13;")
        .replace(
            &format!("contract {}", old_contract_name),
            &format!("contract {}", contract_name),
        );

    // Write updated content to the new file
    fs::write(&renamed_path, updated_content)?;

    // Delete the original file
    fs::remove_file(&verifier_path)?;

    info!(
        LOG,
        "Updated verifier to Solidity 0.8.13 and renamed contract to {}", contract_name
    );

    Ok(())
}

pub async fn deploy_verifier_contract(payload: Payload) -> Result<String> {
    info!(LOG, "Building contracts");
    run_command("yarn", &["build"], None).await?;

    info!(LOG, "Deploying contracts");
    let output = run_command_and_return_output("yarn", &["deploy"], None).await?;

    // Parse the output to extract addresses
    let re =
        Regex::new(r"Deployed (ClientProofVerifier|ServerProofVerifier|Contract|DKIMRegistry) at (0x[a-fA-F0-9]{40})").unwrap();
    let mut contract_addresses = HashMap::new();
    for cap in re.captures_iter(&output) {
        let contract_name = &cap[1];
        let address = &cap[2];
        contract_addresses.insert(contract_name.to_string(), address.to_string());
        info!(LOG, "Deployed {} at address: {}", contract_name, address);
    }

    // Write constructor arguments to a file
    info!(LOG, "Writing constructor arguments to a file");
    let constructor_args = run_command_and_return_output(
        "cast",
        &[
            "abi-encode",
            "constructor(address,address,address)",
            contract_addresses.get("DKIMRegistry").unwrap(),
            contract_addresses.get("ClientProofVerifier").unwrap(),
            contract_addresses.get("ServerProofVerifier").unwrap(),
        ],
        None,
    )
    .await?;

    info!(LOG, "Verify contracts");
    run_command(
        "forge",
        &[
            "verify-contract",
            "--chain-id",
            payload.chain_id.to_string().as_str(),
            contract_addresses.get("ClientProofVerifier").unwrap(),
            "tmp/ClientProofVerifier.sol:ClientProofVerifier",
        ],
        None,
    )
    .await?;

    run_command(
        "forge",
        &[
            "verify-contract",
            "--chain-id",
            payload.chain_id.to_string().as_str(),
            contract_addresses.get("ServerProofVerifier").unwrap(),
            "tmp/ServerProofVerifier.sol:ServerProofVerifier",
        ],
        None,
    )
    .await?;

    run_command(
        "forge",
        &[
            "verify-contract",
            "--chain-id",
            payload.chain_id.to_string().as_str(),
            "--constructor-args",
            &constructor_args,
            contract_addresses.get("Contract").unwrap(),
            "tmp/Contract.sol:Contract",
        ],
        None,
    )
    .await?;

    Ok(contract_addresses.get("Contract").unwrap().to_string())
}
