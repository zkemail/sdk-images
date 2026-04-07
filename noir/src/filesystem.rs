use anyhow::Result;
use relayer_utils::LOG;
use sdk_utils::{run_command, upload_to_url};
use slog::info;
use std::{fs, path::Path};

use crate::handlers::UploadUrls;

#[cfg_attr(test, mockall::automock)]
pub trait FileUploader {
    fn upload_files(&self, upload_urls: UploadUrls) -> impl Future<Output = Result<()>> + Send;
}

pub struct ProductionFileUploader;

impl FileUploader for ProductionFileUploader {
    async fn upload_files(&self, upload_urls: UploadUrls) -> Result<()> {
        // 1024-bit artifacts
        upload_to_url(
            &upload_urls.circuit_1024,
            "./tmp/circuit_1024.zip",
            "application/zip",
        )
        .await?;
        upload_to_url(
            &upload_urls.circuit_json_1024,
            "./tmp/target/sdk_noir_1024.json",
            "application/json",
        )
        .await?;

        // 2048-bit artifacts
        upload_to_url(
            &upload_urls.circuit_2048,
            "./tmp/circuit_2048.zip",
            "application/zip",
        )
        .await?;
        upload_to_url(
            &upload_urls.circuit_json_2048,
            "./tmp/target/sdk_noir_2048.json",
            "application/json",
        )
        .await?;

        // Shared regex graphs
        upload_to_url(
            &upload_urls.regex_graphs,
            "./tmp/regex_graphs.zip",
            "application/zip",
        )
        .await?;

        Ok(())
    }
}

/// Sets up the temporary directory structure for circuit compilation
pub async fn setup() -> Result<()> {
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

    // Ensure src directory exists inside tmp
    let src_path = tmp_path.join("src");

    if src_path.exists() {
        fs::remove_dir_all(&src_path)?;
    }
    fs::create_dir_all(&src_path)?;

    // Copy Nargo.toml to the tmp folder
    let nargo_toml_path = Path::new("./Nargo.toml.txt");

    fs::copy(nargo_toml_path, tmp_path.join("Nargo.toml"))?;

    Ok(())
}

/// Compiles the circuit using nargo and generates the verification key
pub async fn compile_circuit() -> Result<()> {
    // Compile the circuit
    info!(LOG, "Compiling circuit");
    run_command("nargo", &["compile"], Some("tmp")).await?;

    Ok(())
}

/// Cleans up after multi-key compilation and zips both circuit variants
/// Takes the circuit source code for each key size to create separate zips
pub async fn cleanup_multi_key(circuit_1024: &str, circuit_2048: &str) -> Result<()> {
    info!(LOG, "Cleaning up multi-key compilation");

    // Write 1024-bit circuit and zip it
    info!(LOG, "Zipping 1024-bit circuit");
    std::fs::write("./tmp/src/main.nr", circuit_1024)?;
    run_command(
        "zip",
        &["-r", "circuit_1024.zip", "src", "Nargo.toml"],
        Some("tmp"),
    )
    .await?;

    // Write 2048-bit circuit and zip it
    info!(LOG, "Zipping 2048-bit circuit");
    std::fs::write("./tmp/src/main.nr", circuit_2048)?;
    run_command(
        "zip",
        &["-r", "circuit_2048.zip", "src", "Nargo.toml"],
        Some("tmp"),
    )
    .await?;

    // Zip regex graphs (shared)
    info!(LOG, "Zipping regex graphs");
    run_command(
        "zip",
        &["-r", "regex_graphs.zip", ".", "-i", "*_regex.json"],
        Some("tmp"),
    )
    .await?;

    Ok(())
}
