use clap::Parser;
use color_eyre::eyre::Result;
use std::{net::SocketAddr, path::PathBuf};

use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

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

pub fn init_tracing() -> Result<()> {
    let fmt_layer = fmt::layer().compact().with_span_events(
        tracing_subscriber::fmt::format::FmtSpan::NEW
            | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
    );
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");

    let args = Cli::parse();
    load_balancer::run(args.addr, &args.config_file).await?;
    Ok(())
}
