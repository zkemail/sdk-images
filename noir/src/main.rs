use anyhow::Result;
use axum::{Router, routing::post};

use noir::handlers::compile_circuit_handler;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting server");
    let app = Router::new().route("/compile", post(compile_circuit_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
