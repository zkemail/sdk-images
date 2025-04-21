mod db;
mod payload;
mod template;

use std::{fs, path::Path};

use anyhow::Result;
use relayer_utils::LOG;
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

    Ok(())
}
