use anyhow::Result;
use dotenv::dotenv;
use relayer_utils::LOG;
use sdk_utils::{download_file, get_client, run_command, upload_file};
use slog::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

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
        "artifacts/circuit.zip".to_string(),
    )
    .await?;

    compile_circuit("artifacts/circuit.zip").await?;

    upload_file(
        &client,
        bucket,
        blueprint_id,
        "compiled_circuit.zip".to_string(),
        "artifacts/circuit_cpp/compiled_circuit.zip".to_string(),
    )
    .await?;

    Ok(())
}

async fn compile_circuit(circuit_path: &str) -> Result<()> {
    // Unzip circuit files into the artifacts folder
    info!(LOG, "Unzipping circuit");
    run_command("unzip", &["-o", circuit_path, "-d", "artifacts"], None).await?;

    // Run yarn install in the artifacts folder
    info!(LOG, "Running yarn install");
    run_command("yarn", &[], Some("artifacts")).await?;

    // Compile the circuit
    info!(LOG, "Compiling circuit");
    run_command(
        "circom",
        &[
            "circuit.circom",
            "--sym",
            "--r1cs",
            "--c",
            "-l",
            "./node_modules",
        ],
        Some("artifacts"),
    )
    .await?;

    // Run make in the circuit_cpp folder
    info!(LOG, "Running make");
    run_command("make", &[], Some("artifacts/circuit_cpp")).await?;

    // Copy r1cs and sym files to the circuit_cpp folder
    info!(LOG, "Copying r1cs and sym files");
    run_command(
        "cp",
        &["circuit.r1cs", "circuit.sym", "circuit_cpp"],
        Some("artifacts"),
    )
    .await?;

    // Create a zip file with the compiled circuit
    info!(LOG, "Creating zip file");
    run_command(
        "zip",
        &["-r", "compiled_circuit.zip", "."],
        Some("artifacts/circuit_cpp"),
    )
    .await?;

    Ok(())
}
