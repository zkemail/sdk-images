use anyhow::Result;
use dotenv::dotenv;
use rand::Rng;
use regex::Regex;
use relayer_utils::LOG;
use sdk_utils::{
    download_from_url, run_command, run_command_and_return_output, run_command_with_input,
    upload_to_url, Payload,
};
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

    download_from_url(&payload.download_url, "artifacts/compiled_circuit.zip").await?;

    generate_keys("artifacts").await?;

    upload_to_url(&payload.upload_url, "artifacts/keys.zip").await?;

    Ok(())
}

async fn generate_keys(artifact_dir: &str) -> Result<()> {
    // Unzip circuit files into the artifacts folder
    info!(LOG, "Unzipping circuit");
    run_command("unzip", &["-o", "compiled_circuit.zip"], Some(artifact_dir)).await?;

    // Remove compiled_circuit.zip
    run_command("rm", &["compiled_circuit.zip"], Some(artifact_dir)).await?;

    let power = find_power_of_tau(artifact_dir).await?;
    info!(LOG, "Power of tau: {}", power);

    // Start powers of tau ceremony
    run_command(
        "snarkjs",
        &[
            "powersoftau",
            "new",
            "bn128",
            &power.to_string(),
            "pot_0000.ptau",
            "-v",
        ],
        Some(artifact_dir),
    )
    .await?;

    // Contribute to powers of tau ceremony
    info!(LOG, "Contributing to power of tau");
    let random_input: u32 = rand::thread_rng().gen_range(100..1000);
    let random_input_str = format!("{}\n", random_input);
    run_command_with_input(
        "snarkjs",
        &[
            "powersoftau",
            "contribute",
            "pot_0000.ptau",
            "pot_0001.ptau",
            "-v",
        ],
        Some(artifact_dir),
        &random_input_str,
    )
    .await?;

    // Start phase 2 of powers of tau ceremony
    info!(LOG, "Preparing phase 2 of power of tau");
    run_command(
        "snarkjs",
        &[
            "powersoftau",
            "prepare",
            "phase2",
            "pot_0001.ptau",
            "pot_0001_final.ptau",
            "-v",
        ],
        Some(artifact_dir),
    )
    .await?;

    // Generate zkey
    info!(LOG, "Generating zkey");
    run_command(
        "snarkjs",
        &[
            "groth16",
            "setup",
            "circuit.r1cs",
            "pot_0001_final.ptau",
            "circuit_0000.zkey",
        ],
        Some(artifact_dir),
    )
    .await?;

    // Contribute to zkey
    info!(LOG, "Contributing to zkey");
    let random_input: u32 = rand::thread_rng().gen_range(100..1000);
    let random_input_str = format!("{}\n", random_input);
    run_command_with_input(
        "snarkjs",
        &[
            "zkey",
            "contribute",
            "circuit_0000.zkey",
            "circuit.zkey",
            "-v",
        ],
        Some(artifact_dir),
        &random_input_str,
    )
    .await?;

    // Export verification key
    info!(LOG, "Exporting verification key");
    run_command(
        "snarkjs",
        &[
            "zkey",
            "export",
            "verificationkey",
            "circuit.zkey",
            "verification_key.json",
        ],
        Some(artifact_dir),
    )
    .await?;

    // Zip circuit.zkey and verification_key.json
    info!(LOG, "Zipping circuit.zkey and verification_key.json");
    run_command(
        "zip",
        &["keys.zip", "circuit.zkey", "verification_key.json"],
        Some(artifact_dir),
    )
    .await?;

    Ok(())
}

async fn find_power_of_tau(artifact_dir: &str) -> Result<usize> {
    info!(LOG, "Generating power of tau (test)");
    run_command(
        "snarkjs",
        &["powersoftau", "new", "bn128", "1", "pot_0000.ptau", "-v"],
        Some(artifact_dir),
    )
    .await?;

    info!(LOG, "Contributing to power of tau (test)");
    let random_input: u32 = rand::thread_rng().gen_range(100..1000);
    let random_input_str = format!("{}\n", random_input);
    run_command_with_input(
        "snarkjs",
        &[
            "powersoftau",
            "contribute",
            "pot_0000.ptau",
            "pot_0001.ptau",
            "-v",
        ],
        Some(artifact_dir),
        &random_input_str,
    )
    .await?;

    info!(LOG, "Preparing phase 2 of power of tau (test)");
    run_command(
        "snarkjs",
        &[
            "powersoftau",
            "prepare",
            "phase2",
            "pot_0001.ptau",
            "pot_0001_final.ptau",
            "-v",
        ],
        Some(artifact_dir),
    )
    .await?;

    let error_message = run_command_and_return_output(
        "snarkjs",
        &[
            "groth16",
            "setup",
            "circuit.r1cs",
            "pot_0001_final.ptau",
            "circuit.zkey",
        ],
        Some(artifact_dir),
    )
    .await?;
    let re = Regex::new(r"(\d+)\*\d+").unwrap();

    if let Some(captures) = re.captures(&error_message) {
        if let Some(num) = captures.get(1) {
            let num: usize = num.as_str().parse().unwrap();

            let x = (num as f64).log2().ceil() as usize;
            return Ok(x + 1);
        }
    }

    Err(anyhow::Error::msg("Failed to find power of tau"))
}
