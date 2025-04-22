mod db;
mod payload;
mod template;

use std::{fs, path::Path};

use anyhow::Result;
use payload::UploadUrls;
use relayer_utils::LOG;
use sdk_utils::{
    run_command, run_command_and_return_output, run_command_with_input, upload_to_url,
};
use slog::info;
use sqlx::postgres::PgPoolOptions;
use template::{CircuitTemplateInputs, generate_circuit, generate_regex_circuits};

#[tokio::main]
async fn main() -> Result<()> {
    let payload = payload::load_payload()?;
    info!(LOG, "Loaded configuration: {:?}", payload);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&payload.database_url)
        .await?;
    println!("Database connection established");

    let blueprint = payload.clone().blueprint;

    setup().await?;

    // generate_regex_circuits(blueprint.clone().decomposed_regexes)?;

    let circuit_template_inputs = CircuitTemplateInputs::from(blueprint.clone());

    let circuit = generate_circuit(circuit_template_inputs)?;

    // Write the circuit to a file
    let circuit_path = "./tmp/src/main.nr";
    std::fs::write(circuit_path, circuit)?;

    compile_circuit().await?;

    // TODO: Contract deployment

    cleanup().await?;

    // update_verifier_contract_address(&pool, blueprint.id.expect("No ID found"), &contract_address)
    // .await?;

    upload_files(payload.upload_urls).await?;

    Ok(())
}

async fn setup() -> Result<()> {
    // Define the tmp path
    let tmp_path = Path::new("./tmp");

    // If tmp exists, remove its contents
    if tmp_path.exists() {
        for entry in fs::read_dir(tmp_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
    } else {
        // If tmp doesn't exist, create it
        fs::create_dir_all(tmp_path)?;
    }

    // Ensure the regex directory exists inside tmp
    let regex_path = tmp_path.join("regex");
    if regex_path.exists() {
        fs::remove_dir_all(&regex_path)?;
    }
    fs::create_dir_all(&regex_path)?;

    // Ensure src directory exists inside tmp
    let src_path = tmp_path.join("src");
    if src_path.exists() {
        fs::remove_dir_all(&src_path)?;
    }
    fs::create_dir_all(&src_path)?;

    // Copy Nargo.toml to the tmp folder
    let nargo_toml_path = Path::new("./Nargo.toml");
    fs::copy(nargo_toml_path, tmp_path.join("Nargo.toml"))?;

    Ok(())
}

async fn compile_circuit() -> Result<()> {
    // Compile the circuit
    info!(LOG, "Compiling circuit");
    run_command("nargo", &["build"], Some("tmp")).await?;

    // Generate vk
    info!(LOG, "Generating vk");
    run_command(
        "bb",
        &["write_vk", "-b", "./target/sdk_noir.json", "-o", "./vk"],
        Some("tmp"),
    )
    .await?;

    Ok(())
}

async fn cleanup() -> Result<()> {
    info!(LOG, "Cleaning up");

    info!(LOG, "Zipping circuit");
    run_command(
        "zip",
        &["-r", "circuit.zip", "regex", "src", "Nargo.toml"],
        Some("tmp"),
    )
    .await?;

    Ok(())
}

async fn upload_files(upload_urls: UploadUrls) -> Result<()> {
    upload_to_url(&upload_urls.circuit, "./tmp/circuit.zip", "application/zip").await?;
    upload_to_url(
        &upload_urls.circuit_json,
        "./tmp/target/sdk_noir.json",
        "application/json",
    )
    .await?;
    upload_to_url(
        &upload_urls.vk,
        "./tmp/target/vk",
        "application/octet-stream",
    )
    .await?;

    Ok(())
}
