use anyhow::Result;
use axum::{Router, routing::post};

use noir::handlers::compile_circuit_handler;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting server");
    let app = Router::new().route("/compile", post(compile_circuit_handler));

    // Get port from environment variable or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "8082".to_string());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
