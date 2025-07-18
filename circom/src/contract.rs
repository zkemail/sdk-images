use std::{collections::HashMap, env, fs};

use anyhow::Result;
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{run_command, run_command_and_return_output};
use serde::Serialize;
use slog::{info, warn};
use tera::{Context, Tera};
use tokio::time::{sleep, Duration};

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
    tera.add_template_file("./templates/template.sol.tera", Some("contract.sol"))?;

    let mut context = Context::new();
    context.insert("sender_domain", &contract_data.sender_domain);
    context.insert("values", &contract_data.values);
    context.insert("external_inputs", &contract_data.external_inputs);
    context.insert("signal_size", &contract_data.signal_size);
    context.insert(
        "prover_eth_address_idx",
        &contract_data.prover_eth_address_idx,
    );

    let rendered_contract = tera.render("contract.sol", &context)?;

    let re = regex::Regex::new(r"\n+").unwrap();

    let cleaned_contract = re.replace_all(&rendered_contract, "\n").to_string();

    // Write the rendered template to a file
    std::fs::write("tmp/contract.sol", cleaned_contract)?;

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

pub async fn generate_verifier_contract(tmp_dir: &str) -> Result<()> {
    let chunked_snarkjs_path = "../node_modules/.bin/snarkjs";

    // Generate the verifier contract
    info!(LOG, "Generating verifier contract");
    run_command(
        &chunked_snarkjs_path,
        &[
            "zkey",
            "export",
            "solidityverifier",
            "circuit.zkey",
            "verifier.sol",
        ],
        Some(tmp_dir),
    )
    .await?;

    // Update sol version
    let path = "tmp/verifier.sol";

    // Read the file content
    let content = fs::read_to_string(path)?;

    // Replace the pragma line
    let updated_content = content.replace("pragma solidity ^0.6.11;", "pragma solidity ^0.8.13;");

    // Write back to file
    fs::write(path, updated_content)?;

    info!(LOG, "Updated verifier.sol to Solidity 0.8.13");

    Ok(())
}

pub async fn deploy_verifier_contract(payload: Payload) -> Result<String> {
    info!(LOG, "Building contracts");
    run_command("yarn", &["build"], None).await?;

    info!(LOG, "Deploying contracts");
    let output = run_command_and_return_output("yarn", &["deploy"], None).await?;

    // Parse the output to extract addresses
    let re =
        Regex::new(r"Deployed (Verifier|Contract|DKIMRegistry) at (0x[a-fA-F0-9]{40})").unwrap();
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
            "constructor(address,address)",
            contract_addresses.get("DKIMRegistry").unwrap(),
            contract_addresses.get("Verifier").unwrap(),
        ],
        None,
    )
    .await?;

    // if env::var("ETHERSCAN_API_KEY").is_ok() {
    //     info!(LOG, "Verify contracts with retry logic and rate limiting");
        
    //     let chain_id_str = payload.chain_id.to_string();
    //     let max_retries = 3;
    //     let delay_seconds = 3; // Wait 3 seconds between retries (well above 2/sec rate limit)
        
    //     // Verify Verifier contract
    //     verify_contract_with_retry(
    //         &chain_id_str,
    //         contract_addresses.get("Verifier").unwrap(),
    //         "tmp/verifier.sol:Verifier",
    //         None,
    //         max_retries,
    //         delay_seconds,
    //     )
    //     .await?;

    //     // Wait before next verification to respect rate limit
    //     sleep(Duration::from_secs(delay_seconds)).await;

    //     // Verify main Contract with constructor args
    //     verify_contract_with_retry(
    //         &chain_id_str,
    //         contract_addresses.get("Contract").unwrap(),
    //         "tmp/contract.sol:Contract",
    //         Some(&constructor_args),
    //         max_retries,
    //         delay_seconds,
    //     )
    //     .await?;
    // }

    Ok(contract_addresses.get("Contract").unwrap().to_string())
}

async fn verify_contract_with_retry(
    chain_id: &str,
    contract_address: &str,
    contract_path: &str,
    constructor_args: Option<&str>,
    max_retries: u32,
    delay_seconds: u64,
) -> Result<()> {
    let mut args = vec![
        "verify-contract",
        "--chain-id",
        chain_id,
    ];
    
    if let Some(constructor_args) = constructor_args {
        args.push("--constructor-args");
        args.push(constructor_args);
    }
    
    args.push(contract_address);
    args.push(contract_path);

    for attempt in 0..max_retries {
        info!(LOG, "Verifying contract {} (attempt {}/{})", contract_path, attempt + 1, max_retries);
        
        match run_command("forge", &args, None).await {
            Ok(_) => {
                info!(LOG, "Successfully verified contract {}", contract_path);
                return Ok(());
            }
            Err(e) => {
                let error_msg = format!("{:?}", e);
                
                // Check if it's a rate limit error
                if error_msg.contains("rate limit") || error_msg.contains("Max calls per sec") {
                    warn!(LOG, "Rate limit hit for contract {}, attempt {}/{}: {}", 
                          contract_path, attempt + 1, max_retries, error_msg);
                    
                    if attempt < max_retries - 1 {
                        info!(LOG, "Waiting {} seconds before retry...", delay_seconds);
                        sleep(Duration::from_secs(delay_seconds)).await;
                        continue;
                    }
                }
                
                // If it's not a rate limit error or we've exhausted retries, return the error
                warn!(LOG, "Failed to verify contract {} after {} attempts: {}", 
                      contract_path, attempt + 1, error_msg);
                return Err(e);
            }
        }
    }
    
    Ok(())
}
