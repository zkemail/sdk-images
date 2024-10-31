mod contract;
mod db;
mod payload;

use anyhow::Result;
use contract::{
    create_contract, deploy_verifier_contract, generate_verifier_contract, prepare_contract_data,
};
use db::update_verifier_contract_address;
use relayer_utils::LOG;
use sdk_utils::{download_from_url, run_command, upload_to_url};
use slog::info;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    let payload = payload::load_payload()?;
    info!(LOG, "Loaded configuration: {:?}", payload);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&payload.database_url)
        .await?;
    println!("Database connection established");

    // Create an artifact folder if it doesn't exist
    std::fs::create_dir_all("tmp")?;

    download_from_url(&payload.download_url, "tmp/keys.zip").await?;

    generate_verifier_contract("tmp").await?;

    create_contract(&prepare_contract_data(&payload))?;

    info!(LOG, "Deploying verifier contract");
    let verifier_contract_address = deploy_verifier_contract(payload.clone()).await?;
    update_verifier_contract_address(
        &pool,
        payload.blueprint.id.expect("Blueprint ID not found"),
        &verifier_contract_address,
    )
    .await?;

    // Clean up the artifacts folder
    info!(LOG, "Cleaning up artifacts");
    clean_up_artifacts("tmp").await?;

    // Upload the complete circuit to the specified URL
    upload_to_url(&payload.upload_url, "tmp/verifier_contract.zip").await?;

    Ok(())
}

async fn clean_up_artifacts(artifact_dir: &str) -> Result<()> {
    // Ensure the artifacts/src directory exists
    run_command("mkdir", &["-p", "tmp/src"], None).await?;

    // Copy the src/contract.sol and src/verifier.sol file to a new src folder in the artifacts directory
    run_command(
        "mv",
        &[
            "src/contract.sol",
            "src/verifier.sol",
            format!("{}/src/", artifact_dir).as_str(),
        ],
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
            format!("{}/", artifact_dir).as_str(),
        ],
        None,
    )
    .await?;

    // Delete circuit.zkey, contract.sol, verifier.sol, and keys.zip
    run_command(
        "rm",
        &[
            "circuit.zkey",
            "keys.zip",
            "verification_key.json",
            "verifier.sol",
        ],
        Some(artifact_dir),
    )
    .await?;

    // Zip the artifacts folder
    run_command(
        "zip",
        &["-r", "verifier_contract.zip", "."],
        Some(artifact_dir),
    )
    .await?;

    Ok(())
}
