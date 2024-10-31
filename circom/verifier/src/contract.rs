use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use handlebars::Handlebars;
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{run_command, run_command_and_return_output};
use serde::{Deserialize, Serialize};
use slog::info;
use tera::{Context, Tera};

use crate::payload::Payload;

#[derive(Serialize)]
pub struct ContractData {
    pub sender_domain: String,
    pub values: Vec<Field>,
    pub external_inputs: Vec<Field>,
    pub signal_size: usize,
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
    tera.add_template_file("./template/contract.sol.tera", Some("contract.sol"))?;

    let mut context = Context::new();
    context.insert("sender_domain", &contract_data.sender_domain);
    context.insert("values", &contract_data.values);
    context.insert("external_inputs", &contract_data.external_inputs);
    context.insert("signal_size", &contract_data.signal_size);

    let rendered_contract = tera.render("contract.sol", &context)?;

    // Write the rendered template to a file
    std::fs::write("src/contract.sol", rendered_contract)?;

    Ok(())
}

pub fn prepare_contract_data(payload: &Payload) -> ContractData {
    let mut signal_size = 1; // Start with 1 as per your logic
    let mut current_idx = 1; // Start index for the first field

    let mut values = Vec::new();
    for regex in payload.blueprint.decomposed_regexes.iter() {
        let pack_size = ((regex.max_length as f64) / 31.0).ceil() as usize;
        let field = Field {
            name: regex.name.clone(),
            max_length: regex.max_length,
            pack_size,
            start_idx: current_idx,
        };
        signal_size += pack_size;
        current_idx += pack_size;
        values.push(field);
    }

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
    }
}

pub async fn generate_verifier_contract(artifact_dir: &str) -> Result<()> {
    // Unzip circuit files into the artifacts folder
    info!(LOG, "Unzipping circuit");
    run_command("unzip", &["-o", "keys.zip"], Some(artifact_dir)).await?;

    // Generate the verifier contract
    info!(LOG, "Generating verifier contract");
    run_command(
        "snarkjs",
        &[
            "zkey",
            "export",
            "solidityverifier",
            "circuit.zkey",
            "verifier.sol",
        ],
        Some(artifact_dir),
    )
    .await?;

    // Copy the verifier contract to the src folder
    info!(LOG, "Copying verifier contract");
    run_command(
        "cp",
        &["verifier.sol", "../src/verifier.sol"],
        Some(artifact_dir),
    )
    .await?;

    Ok(())
}

pub async fn deploy_verifier_contract(payload: Payload) -> Result<String> {
    info!(LOG, "Building contracts");
    run_command("yarn", &["build"], None).await?;

    info!(LOG, "Deploying contracts");
    let output = run_command_and_return_output("yarn", &["deploy"], None).await?;

    // Parse the output to extract addresses
    let re = Regex::new(r"Deployed (Groth16Verifier|Contract|DKIMRegistry) at (0x[a-fA-F0-9]{40})")
        .unwrap();
    let mut contract_addresses = HashMap::new();
    for cap in re.captures_iter(&output) {
        let contract_name = &cap[1];
        let address = &cap[2];
        contract_addresses.insert(contract_name.to_string(), address.to_string());
        info!(LOG, "Deployed {} at address: {}", contract_name, address);
    }

    info!(LOG, "Verify contracts");
    run_command(
        "forge",
        &[
            "verify-contract",
            "--chain-id",
            payload.chain_id.to_string().as_str(),
            contract_addresses.get("Groth16Verifier").unwrap(),
            "src/verifier.sol:Groth16Verifier",
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
            contract_addresses.get("Contract").unwrap(),
            "src/contract.sol:Contract",
        ],
        None,
    )
    .await?;

    Ok(contract_addresses.get("Contract").unwrap().to_string())
}
