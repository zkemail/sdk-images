use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use handlebars::Handlebars;
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{run_command, run_command_and_return_output};
use serde::{Deserialize, Serialize};
use slog::info;

use crate::config::Payload;

#[derive(Serialize)]
pub struct ContractData {
    pub sender_domain: String,
    pub values: Vec<Field>,
    pub external_inputs: Vec<Field>,
}

#[derive(Serialize)]
pub struct Field {
    pub name: String,
    pub max_length: usize,
}

pub fn create_contract(contract_data: ContractData) -> Result<()> {
    // Initialize Handlebars
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("contract", include_str!("../templates/contract.hbs"))?;

    // Render the template with the data
    let rendered = handlebars.render("contract", &contract_data)?;

    // Write the rendered template to a file
    std::fs::write("src/contract.sol", rendered)?;

    Ok(())
}

pub async fn generate_verifier_contract(artifact_dir: &str) -> Result<()> {
    // Unzip circuit files into the artifacts folder
    info!(LOG, "Unzipping circuit");
    run_command(
        "unzip",
        &["-o", "compiled_circuit_with_keys.zip"],
        Some(artifact_dir),
    )
    .await?;

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

pub async fn deploy_verifier_contract(payload: Payload) -> Result<()> {
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

    Ok(())
}
