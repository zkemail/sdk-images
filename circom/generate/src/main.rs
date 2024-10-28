mod blueprint;
mod template;

use blueprint::Payload;

use anyhow::Result;
use dotenv::dotenv;
use sdk_utils::{run_command, upload_to_url};
use template::{generate_circuit, generate_regex_circuits, CircuitTemplateInputs};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let payload: Payload = serde_json::from_str(
        std::env::var("PAYLOAD")
            .expect("PAYLOAD environment variable not set")
            .as_str(),
    )?;

    let blueprint = payload.blueprint;
    let upload_url = payload.upload_url;

    setup().await?;

    generate_regex_circuits(blueprint.clone().decomposed_regexes)?;

    let circuit_template_inputs = CircuitTemplateInputs::from(blueprint);

    let circuit = generate_circuit(circuit_template_inputs)?;

    // Write the circuit to a file
    let circuit_path = "./circuit.circom";
    std::fs::write(circuit_path, circuit)?;

    cleanup().await?;

    upload_to_url(&upload_url, "./artifacts/circuit.zip").await?;

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
    run_command(
        "cp",
        &[
            "package.json",
            "artifacts/package.json",
            "yarn.lock",
            "artifacts/yarn.lock",
        ],
        None,
    )
    .await?;

    // Zip everything in artifacts
    run_command("zip", &["-r", "circuit.zip", "."], Some("artifacts")).await?;

    Ok(())
}
