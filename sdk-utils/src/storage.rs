use anyhow::Result;
use relayer_utils::LOG;
use slog::info;

pub async fn upload_to_url(upload_url: &str, file_path: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let file = std::fs::read(file_path)?;

    let response = client.post(upload_url).body(file).send().await?;

    if response.status().is_success() {
        info!(LOG, "File uploaded successfully");
    } else {
        info!(LOG, "Failed to upload file");
    }

    Ok(())
}
