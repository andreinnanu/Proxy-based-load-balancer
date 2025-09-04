use std::net::SocketAddr;

use hyper::server::conn::http1;

use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::net::TcpListener;
use tower::ServiceBuilder;

use crate::{middleware::Logger, services::LoadBalancer};

pub mod middleware;
pub mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::spawn(async move {
            let load_balancer_tower_service = ServiceBuilder::new()
                .layer_fn(Logger::new)
                .service(LoadBalancer::new(3));

            let hyper_service = TowerToHyperService::new(load_balancer_tower_service);

            if let Err(err) = http1::Builder::new()
                .serve_connection(io, hyper_service)
                .await
            {
                eprintln!("server error: {err}");
            }
        });
    }
}
