pub mod least_connections;
pub mod round_robin;

pub use round_robin::RoundRobin;
pub use least_connections::LeastConnections;

use std::{collections::HashMap, net::SocketAddr};

use crate::services::HostStatus;

pub enum Strategy {
    RoundRobin,
}

pub trait Algorithm: Send + Sync {
    fn get_host(&mut self, hosts: &mut HashMap<SocketAddr, HostStatus>) -> SocketAddr;
}