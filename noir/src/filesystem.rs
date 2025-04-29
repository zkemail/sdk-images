use anyhow::Result;
use relayer_utils::LOG;
use sdk_utils::{run_command, upload_to_url};
use slog::info;
use std::{fs, path::Path};

use crate::handlers::UploadUrls;

/// Sets up the temporary directory structure for circuit compilation
pub async fn setup() -> Result<()> {
    // Define the tmp path
    let tmp_path = Path::new("./tmp");
    println!("tmp_path: {:?}", tmp_path);

    // If tmp exists, remove its contents
    if tmp_path.exists() {
        println!("tmp_path does not exist");
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
        println!("creating tmp dir");
        // If tmp doesn't exist, create it
        fs::create_dir_all(tmp_path)?;
    }

    println!("created all temp dir stuff");

    // Ensure src directory exists inside tmp
    let src_path = tmp_path.join("src");
    println!("src_path: {:?}", src_path);

    if src_path.exists() {
        fs::remove_dir_all(&src_path)?;
    }
    fs::create_dir_all(&src_path)?;
    println!("create_dir_all after create_dir_all");

    // Copy Nargo.toml to the tmp folder
    let nargo_toml_path = Path::new("./Nargo.toml");
    println!("nargo_toml_path: {:?}", nargo_toml_path);

    fs::copy(nargo_toml_path, tmp_path.join("Nargo.toml"))?;
    println!("copied nargo");

    Ok(())
}

/// Compiles the circuit using nargo and generates the verification key
pub async fn compile_circuit() -> Result<()> {
    // Compile the circuit
    info!(LOG, "Compiling circuit");
    run_command("nargo", &["compile"], Some("tmp")).await?;

    Ok(())
}

/// Cleans up after compilation and zips the circuit files
pub async fn cleanup() -> Result<()> {
    info!(LOG, "Cleaning up");

    info!(LOG, "Zipping circuit");
    run_command(
        "zip",
        &["-r", "circuit.zip", "src", "Nargo.toml"],
        Some("tmp"),
    )
    .await?;

    info!(LOG, "Zipping regex graphs");
    run_command(
        "zip",
        &["-r", "regex_graphs.zip", ".", "-i", "*_regex.json"],
        Some("tmp"),
    )
    .await?;

    Ok(())
}

/// Uploads the compiled circuit files to the provided URLs
pub async fn upload_files(upload_urls: UploadUrls) -> Result<()> {
    upload_to_url(&upload_urls.circuit, "./tmp/circuit.zip", "application/zip").await?;
    upload_to_url(
        &upload_urls.circuit_json,
        "./tmp/target/sdk_noir.json",
        "application/json",
    )
    .await?;
    upload_to_url(
        &upload_urls.regex_graphs,
        "./tmp/regex_graphs.zip",
        "application/zip",
    )
    .await?;

    Ok(())
}
