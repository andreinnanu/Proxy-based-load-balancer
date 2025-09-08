use load_balancer::services::LoadBalancer;
use std::net::SocketAddr;

pub mod middleware;
pub mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    load_balancer::run(addr, LoadBalancer::new(3)).await?;
    Ok(())
}
