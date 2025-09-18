use color_eyre::eyre::Result;
use hyper::server::conn::http1;
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use tower::ServiceBuilder;

use crate::{
    middleware::Logger,
    services::{LoadBalancer, LoadBalancerState},
};

pub mod middleware;
pub mod services;
pub mod utils;

pub async fn run(addr: SocketAddr, config_file: &PathBuf) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let load_balancer =
        LoadBalancer::new(Arc::new(RwLock::new(LoadBalancerState::new(config_file))));

    load_balancer.spawn_health_check_task().await;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let load_balancer_clone = load_balancer.clone();

        tokio::spawn(async move {
            let load_balancer_tower_service = ServiceBuilder::new()
                .layer_fn(Logger::new)
                .service(load_balancer_clone);

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
