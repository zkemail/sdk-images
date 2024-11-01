use anyhow::Result;
use dotenv::dotenv;
use relayer_utils::LOG;
use sdk_utils::{download_from_url, run_command, upload_to_url};
use serde::{Deserialize, Serialize};
use slog::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub input_download_url: String,
    pub keys_download_url: String,
    pub compiled_circuit_download_url: String,
    pub proof_upload_url: String,
    pub public_upload_url: String,
}

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

    download_from_url(&payload.input_download_url, "artifacts/input.json").await?;
    download_from_url(&payload.keys_download_url, "artifacts/keys.zip").await?;
    download_from_url(
        &payload.compiled_circuit_download_url,
        "artifacts/compiled_circuit.zip",
    )
    .await?;

    prove("artifacts").await?;

    upload_to_url(&payload.proof_upload_url, "artifacts/proof.json").await?;
    upload_to_url(&payload.public_upload_url, "artifacts/public.json").await?;

    Ok(())
}

async fn prove(artifacts_dir: &str) -> Result<()> {
    // Unzip keys files into the artifacts folder
    info!(LOG, "Unzipping keys");
    run_command("unzip", &["-o", "keys.zip"], Some(artifacts_dir)).await?;

    // Generate witness
    info!(LOG, "Generating witness");
    run_command(
        "./circuit",
        &["input.json", "witness.wtns"],
        Some("artifacts"),
    )
    .await?;

    // Generate the proof
    info!(LOG, "Generating proof");
    run_command(
        "snarkjs",
        &[
            "groth16",
            "prove",
            "circuit.zkey",
            "witness.wtns",
            "proof.json",
            "public.json",
        ],
        Some("artifacts"),
    )
    .await?;

    Ok(())
}
