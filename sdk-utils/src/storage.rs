use anyhow::Result;
use relayer_utils::LOG;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use slog::info;

#[derive(Deserialize)]
pub struct Payload {
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    #[serde(rename = "uploadUrl")]
    pub upload_url: String,
}

pub async fn download_from_url(download_url: &str, file_path: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(download_url).send().await?;

    if response.status().is_success() {
        let file = response.bytes().await?;
        std::fs::write(file_path, file)?;
        info!(LOG, "File downloaded successfully");
    } else {
        info!(LOG, "Failed to download file");
    }

    Ok(())
}

pub async fn upload_to_url(upload_url: &str, file_path: &str, file_type: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let file = std::fs::read(file_path)?;

    let response = client
        .put(upload_url)
        .header(CONTENT_TYPE, file_type)
        .body(file)
        .send()
        .await?;

    if response.status().is_success() {
        info!(LOG, "File uploaded successfully");
    } else {
        info!(LOG, "Failed to upload file");
    }

    Ok(())
}
