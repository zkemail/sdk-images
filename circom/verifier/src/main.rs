mod config;
mod contract;

use std::collections::HashMap;

use anyhow::Result;
use config::Config;
use contract::{
    create_contract, deploy_verifier_contract, generate_verifier_contract, ContractData, Field,
};
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{
    download_file, get_client, run_command, run_command_and_return_output, upload_file,
};
use slog::info;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::load_config()?;
    info!(LOG, "Loaded configuration: {:?}", config);

    let client = get_client().await?;

    // Read bucket and object from env
    let object = format!("{}/circuit.zip", config.blueprint_id);

    // Create an artifact folder if it doesn't exist
    std::fs::create_dir_all("artifacts")?;

    download_file(
        &client,
        config.bucket.clone(),
        object,
        "artifacts/compiled_circuit_with_keys.zip".to_string(),
    )
    .await?;

    generate_verifier_contract("artifacts").await?;

    create_contract(ContractData {
        sender_domain: "example.com".to_string(),
        values: vec![
            Field {
                name: "value1".to_string(),
                max_length: 32,
            },
            Field {
                name: "value2".to_string(),
                max_length: 32,
            },
        ],
        external_inputs: vec![Field {
            name: "external_input".to_string(),
            max_length: 32,
        }],
    })?;

    info!(LOG, "Deploying verifier contract");
    deploy_verifier_contract(config.clone()).await?;

    // Clean up the artifacts folder
    info!(LOG, "Cleaning up artifacts");
    clean_up_artifacts("artifacts").await?;

    upload_file(
        &client,
        config.bucket,
        config.blueprint_id,
        "complete_circuit.zip".to_string(),
        "artifacts/complete_circuit.zip".to_string(),
    )
    .await?;

    Ok(())
}

async fn clean_up_artifacts(artifact_dir: &str) -> Result<()> {
    // Ensure the artifacts/src directory exists
    run_command("mkdir", &["-p", "artifacts/src"], None).await?;

    // Copy the src/contract.sol and src/verifier.sol file to a new src folder in the artifacts directory
    run_command(
        "cp",
        &["src/contract.sol", "src/verifier.sol", "artifacts/src/"],
        None,
    )
    .await?;

    // Copy node_modules to the artifacts folder
    run_command(
        "cp",
        &["-r", "node_modules", "artifacts/node_modules"],
        None,
    )
    .await?;

    // Copy Deply.s.sol, foundry.toml, package.json, remappings.txt, and yarn.lock to the artifacts folder
    run_command(
        "cp",
        &[
            "Deploy.s.sol",
            "foundry.toml",
            "package.json",
            "remappings.txt",
            "yarn.lock",
            "artifacts/",
        ],
        None,
    )
    .await?;

    // Zip the artifacts folder
    run_command(
        "zip",
        &["-r", "complete_circuit.zip", "."],
        Some(artifact_dir),
    )
    .await?;

    Ok(())
}
