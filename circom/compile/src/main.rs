use anyhow::Result;
use dotenv::dotenv;
use relayer_utils::LOG;
use sdk_utils::{download_from_url, run_command, upload_to_url, Payload};
use slog::info;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let payload: Payload = serde_json::from_str(
        std::env::var("PAYLOAD")
            .expect("PAYLOAD environment variable not set")
            .as_str(),
    )?;

    // Create an artifact folder if it doesn't exist
    std::fs::create_dir_all("artifacts")?;

    download_from_url(&payload.download_url, "artifacts/circuit.zip").await?;

    compile_circuit("artifacts/circuit.zip").await?;

    upload_to_url(&payload.upload_url, "artifacts/compiled_circuit.zip").await?;

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
    let num_procs = num_cpus::get().to_string();
    run_command("make", &["-j", &num_procs], Some("artifacts/circuit_cpp")).await?;

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
