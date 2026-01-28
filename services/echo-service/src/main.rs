use axum::{routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
struct EchoRequest {
    message: String,
}

#[derive(Debug, Serialize)]
struct EchoResponse {
    reply: String,
}

#[tokio::main]
async fn main() {
    common::init_tracing();

    let app = Router::new()
        .route("/echo", post(echo))
        .route("/health", get(health));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    tracing::info!("echo-service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn echo(Json(payload): Json<EchoRequest>) -> Json<EchoResponse> {
    Json(EchoResponse {
        reply: format!("Hello {}", payload.message),
    })
}

async fn health() -> &'static str {
    "OK"
}
