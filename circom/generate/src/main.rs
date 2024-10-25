mod blueprint;
mod template;

use blueprint::get_blueprint;
use sqlx::postgres::PgPoolOptions;

use anyhow::Result;
use dotenv::dotenv;
use sdk_utils::{get_client, run_command, upload_file};
use template::{generate_circuit, generate_regex_circuits, CircuitTemplateInputs};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let client = get_client().await?;

    let bucket = std::env::var("BUCKET")?;
    let database_url = std::env::var("DATABASE_URL")?;
    let blueprint_id = std::env::var("BLUEPRINT_ID")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    println!("Database connection established");

    setup().await?;

    let blueprint =
        get_blueprint(&pool, Uuid::parse_str(&blueprint_id).expect("Invalid UUID")).await?;

    generate_regex_circuits(blueprint.clone().decomposed_regexes)?;

    let circuit_template_inputs = CircuitTemplateInputs::from(blueprint);

    let circuit = generate_circuit(circuit_template_inputs)?;

    // Write the circuit to a file
    let circuit_path = "./circuit.circom";
    std::fs::write(circuit_path, circuit)?;

    cleanup().await?;

    upload_file(
        &client,
        bucket,
        blueprint_id,
        "circuit.zip".to_string(),
        "artifacts/circuit.zip".to_string(),
    )
    .await?;

    Ok(())
}

async fn setup() -> Result<()> {
    // Remove and recreate artifacts and regex directories
    if std::path::Path::new("./artifacts").exists() {
        std::fs::remove_dir_all("./artifacts")?;
    }
    std::fs::create_dir_all("./artifacts")?;

    if std::path::Path::new("./regex").exists() {
        std::fs::remove_dir_all("./regex")?;
    }
    std::fs::create_dir_all("./regex")?;

    Ok(())
}

async fn cleanup() -> Result<()> {
    // Move circuit.circom to artifacts
    run_command("mv", &["circuit.circom", "artifacts/circuit.circom"], None).await?;

    // Move regex directory to artifacts
    run_command("mv", &["regex", "artifacts/regex"], None).await?;

    // Move package.json and yarn.lock to artifacts
    run_command("cp", &["package.json", "artifacts/package.json"], None).await?;

    // Zip everything in artifacts
    run_command("zip", &["-r", "circuit.zip", "."], Some("artifacts")).await?;

    Ok(())
}
