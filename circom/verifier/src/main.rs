mod chain;
mod config;

use anyhow::Result;
use chain::{create_contract, ContractData, Field};
use relayer_utils::LOG;
use sdk_utils::{download_file, get_client, run_command, upload_file};
use slog::info;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::load_config()?;
    info!(LOG, "Loaded configuration: {:?}", config);

    let client = get_client().await?;

    // Read bucket and object from env
    let bucket = std::env::var("BUCKET")?;
    let blueprint_id = std::env::var("BLUEPRINT_ID")?;
    let object = format!("{}/circuit.zip", blueprint_id);

    // Create an artifact folder if it doesn't exist
    std::fs::create_dir_all("artifacts")?;

    download_file(
        &client,
        bucket.clone(),
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

    upload_file(
        &client,
        bucket,
        blueprint_id,
        "artifacts/complete_circuit.zip".to_string(),
    )
    .await?;

    Ok(())
}

async fn generate_verifier_contract(artifact_dir: &str) -> Result<()> {
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

    Ok(())
}
