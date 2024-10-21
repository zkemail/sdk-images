mod circuit;
mod storage;

use anyhow::Result;
use circuit::compile_circuit;
use dotenv::dotenv;
use storage::{download_file, upload_file};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    println!("Hello, world!");

    let client = storage::get_client().await?;

    // Read bucket and object from env
    let bucket = std::env::var("BUCKET")?;
    let object = format!("{}/circuit.zip", std::env::var("BLUEPRINT_ID")?);

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
        "artifacts/compiled_circuit.zip".to_string(),
    )
    .await?;

    Ok(())
}
