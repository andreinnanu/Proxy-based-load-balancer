use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};

pub mod middleware;
pub mod services;
pub mod utils;

#[derive(Parser, Debug)]
#[command(about = "Runs the load balancer on the specified address", long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = SocketAddr::from(([127, 0, 0, 1], 3000)))]
    addr: SocketAddr,
    #[arg(short, long, required = true)]
    config_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    color_eyre::install().expect("Failed to install color_eyre");

    let args = Cli::parse();
    load_balancer::run(args.addr, &args.config_file).await?;
    Ok(())
}
