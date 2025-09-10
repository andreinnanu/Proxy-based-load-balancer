use std::time::Duration;

use axum::{Router, extract::Query, http::StatusCode, response::IntoResponse, routing::get};
use serde::Deserialize;

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

#[derive(Deserialize, Debug)]
struct WorkParams {
    duration_millis: Option<u64>,
}

async fn work(Query(params): Query<WorkParams>) -> impl IntoResponse {
    tokio::time::sleep(Duration::from_millis(params.duration_millis.unwrap_or(10))).await;
    (StatusCode::OK, "worked hard")
}
