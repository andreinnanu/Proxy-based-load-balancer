use std::{collections::HashMap, net::SocketAddr};

use crate::services::{Algorithm, HostStatus};

#[derive(Clone, Debug, Default)]
pub struct LeastConnections;

impl Algorithm for LeastConnections {
    fn get_host(&mut self, hosts: &mut HashMap<SocketAddr, HostStatus>) -> SocketAddr {
        if let Some((host, x)) = hosts
            .iter()
            .filter(|&(_host, host_status)| host_status.healthy)
            .min_by_key(|(_host, status)| status.open_connections)
        {
            println!("Host: {} with {} connections", host, x.open_connections);
            *host
        } else {
            panic!("No healthy hosts");
        }
    }
}
