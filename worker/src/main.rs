use std::time::Duration;

use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

const ADDRESS: &str = "0.0.0.0:7777";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/work", get(work));

    let listener = tokio::net::TcpListener::bind(ADDRESS).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "healthy")
}

async fn work() -> impl IntoResponse {
    tokio::time::sleep(Duration::from_millis(10)).await;
    (StatusCode::OK, "worked hard")
}
