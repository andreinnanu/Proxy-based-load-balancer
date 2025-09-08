use clap::Parser;
use load_balancer::services::LoadBalancer;
use std::net::SocketAddr;

pub mod middleware;
pub mod services;

#[derive(Parser, Debug)]
#[command(about = "Runs the load balancer on the specified address", long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = SocketAddr::from(([127, 0, 0, 1], 3000)))]
    addr: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    color_eyre::install().expect("Failed to install color_eyre");

    let args = Cli::parse();
    load_balancer::run(args.addr, LoadBalancer::new(3)).await?;
    Ok(())
}
