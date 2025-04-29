use anyhow::Result;
use axum::{Router, middleware, routing::post};
use dotenv::dotenv;
use relayer_utils::LOG;
use slog::warn;

use noir::handlers::compile_circuit_handler;

#[derive(Clone)]
struct AppState {
    api_key: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    println!("Starting server");

    let state = AppState {
        api_key: std::env::var("ZKEMAIL_API_KEY").expect("ZKEMAIL_API_KEY must be set"),
    };

    // Middleware to check API key
    async fn auth_middleware(
        state: axum::extract::State<AppState>,
        req: axum::extract::Request,
        next: middleware::Next,
    ) -> Result<axum::response::Response, axum::http::StatusCode> {
        let query = req.uri().query().unwrap_or("");

        if query
            .split('&')
            .any(|param| param == format!("api_key={}", state.api_key))
        {
            Ok(next.run(req).await)
        } else {
            warn!(LOG, "Unauthorized request attempt");
            Err(axum::http::StatusCode::UNAUTHORIZED)
        }
    }

    let app = Router::new()
        .route("/compile", post(compile_circuit_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // Get port from environment variable or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "8082".to_string());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
