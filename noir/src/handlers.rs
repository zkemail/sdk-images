use anyhow::Result;
use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use relayer_utils::LOG;
use sdk_utils::Blueprint;
use serde::Deserialize;
use slog::info;

// Import from the crate root
use crate::circuit_generator::generate_circuit;
use crate::filesystem::{cleanup, compile_circuit, setup, upload_files};
use crate::models::CircuitTemplateInputs;
use crate::regex_generator::generate_regex_circuits;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrls {
    pub circuit: String,
    pub circuit_json: String,
    pub regex_graphs: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub blueprint: Blueprint,
    pub upload_urls: UploadUrls,
    pub database_url: String,
    pub private_key: String,
    pub rpc_url: String,
    pub chain_id: u32,
    pub etherscan_api_key: String,
    pub dkim_registry_address: String,
}

pub async fn compile_circuit_handler(
    Json(payload): Json<Payload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!(LOG, "Received payload: {:?}", payload);
    println!("payload: {:?}", payload);

    // Process the request
    match process_circuit(payload).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn process_circuit(payload: Payload) -> Result<()> {
    // Setup filesystem
    println!("setting up");
    setup().await?;

    // Extract blueprint
    let blueprint = payload.blueprint;

    println!("generate_regex_circuits");
    // Generate regex circuits
    generate_regex_circuits(&blueprint.decomposed_regexes)?;

    // Generate main circuit from template
    let circuit_template_inputs = CircuitTemplateInputs::from(blueprint);

    let circuit = generate_circuit(circuit_template_inputs)?;
    println!("circuit: {:?}", circuit);

    // Write the circuit to a file
    let circuit_path = "./tmp/src/main.nr";
    std::fs::write(circuit_path, circuit)?;
    println!("wrote circuit_path");

    // Compile and clean up
    compile_circuit().await?;
    println!("compiling done");
    cleanup().await?;
    println!("cleanup done");

    // Upload files
    upload_files(payload.upload_urls).await?;

    Ok(())
}
