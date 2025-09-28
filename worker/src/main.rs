use std::time::Duration;

use axum::{Router, extract::Query, http::StatusCode, response::IntoResponse, routing::get};
use serde::Deserialize;

const ADDRESS: &str = "0.0.0.0:7777";
const WORK_DURATION_MS: &str = "WORK_DURATION_MS";

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
    let work_duration = match params.duration_millis {
        Some(duration) => duration,
        None => std::env::var(WORK_DURATION_MS)
            .unwrap_or("10".to_owned())
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("{WORK_DURATION_MS} to be u64")),
    };
    tokio::time::sleep(Duration::from_millis(
        params.duration_millis.unwrap_or(work_duration),
    ))
    .await;
    (StatusCode::OK, "worked hard")
}
