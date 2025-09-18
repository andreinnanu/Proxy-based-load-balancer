pub mod least_connections;
pub mod round_robin;

pub use least_connections::LeastConnections;
pub use round_robin::RoundRobin;

use std::{collections::HashMap, net::SocketAddr};
use strum_macros::{AsRefStr, EnumString};

use crate::services::HostStatus;

#[derive(AsRefStr, EnumString)]
pub enum Strategy {
    RoundRobin,
    LeastConnections,
}

pub trait Algorithm: Send + Sync {
    fn get_host(&mut self, hosts: &mut HashMap<SocketAddr, HostStatus>) -> Option<SocketAddr>;
}
