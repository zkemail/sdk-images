use anyhow::Result;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        download::Range,
        get::GetObjectRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
};
use relayer_utils::LOG;
use reqwest_middleware::{reqwest, ClientBuilder};
use reqwest_retry::{policies::ExponentialBackoff, Jitter, RetryTransientMiddleware};
use slog::info;

pub async fn get_client() -> Result<Client> {
    let retry_policy = ExponentialBackoff::builder()
        .base(2)
        .jitter(Jitter::Full)
        .build_with_max_retries(3);

    let mid_client = ClientBuilder::new(reqwest::Client::default())
        // reqwest-retry already comes with a default retry stategy that matches http standards
        // override it only if you need a custom one due to non standard behaviour
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    Ok(Client::new(
        ClientConfig {
            http: Some(mid_client),
            ..Default::default()
        }
        .with_auth()
        .await?,
    ))
}

pub async fn download_file(
    client: &Client,
    bucket: String,
    object: String,
    download_path: String,
) -> Result<()> {
    let data = client
        .download_object(
            &GetObjectRequest {
                bucket,
                object,
                ..Default::default()
            },
            &Range::default(),
        )
        .await?;

    info!(LOG, "Download path: {}", download_path);
    std::fs::write(download_path, data).expect("Unable to write file");

    Ok(())
}

pub async fn upload_file(
    client: &Client,
    bucket: String,
    blueprint_id: String,
    name: String,
    file_path: String,
) -> Result<()> {
    let data = std::fs::read(file_path).expect("Unable to read file");

    info!(LOG, "Uploading file");
    let upload_type = UploadType::Simple(Media {
        name: format!("{}/{}", blueprint_id, name).into(),
        content_type: "application/zip".to_string().into(),
        content_length: None,
    });
    client
        .upload_object(
            &UploadObjectRequest {
                bucket,
                ..Default::default()
            },
            data,
            &upload_type,
        )
        .await?;

    Ok(())
}
