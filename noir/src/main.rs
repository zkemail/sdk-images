use anyhow::Result;
use relayer_utils::LOG;
use slog::info;
use sqlx::postgres::PgPoolOptions;

use noir::{
    CircuitTemplateInputs,
    filesystem::{cleanup, compile_circuit, setup, upload_files},
    generate_circuit, generate_regex_circuits,
    payload::load_payload,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let payload = load_payload()?;
    info!(LOG, "Loaded configuration: {:?}", payload);

    // // Connect to database
    // let pool = PgPoolOptions::new()
    //     .max_connections(10)
    //     .connect(&payload.database_url)
    //     .await?;
    // println!("Database connection established");

    // Setup filesystem
    setup().await?;

    // Extract blueprint
    let blueprint = payload.blueprint;

    // Generate regex circuits
    generate_regex_circuits(&blueprint.decomposed_regexes)?;

    // Generate main circuit from template
    let circuit_template_inputs = CircuitTemplateInputs::from(blueprint);
    let circuit = generate_circuit(circuit_template_inputs)?;

    // Write the circuit to a file
    let circuit_path = "./tmp/src/main.nr";
    std::fs::write(circuit_path, circuit)?;

    // Compile and clean up
    compile_circuit().await?;
    cleanup().await?;

    // Upload files
    upload_files(payload.upload_urls).await?;

    Ok(())
}
