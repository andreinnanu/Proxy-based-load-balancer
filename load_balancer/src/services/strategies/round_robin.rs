use std::{collections::HashMap, net::SocketAddr};

use crate::services::{Algorithm, HostStatus};

#[derive(Clone, Debug, Default)]
pub struct RoundRobin {
    last_host: usize,
    hosts_vec: Vec<SocketAddr>,
}

impl RoundRobin {
    pub fn new(hosts: &mut HashMap<SocketAddr, HostStatus>) -> Self {
        let mut hosts_vec = Vec::new();
        for host in hosts.keys() {
            hosts_vec.push(*host);
        }

        Self {
            hosts_vec,
            last_host: 0,
        }
    }
}

impl Algorithm for RoundRobin {
    fn get_host(&mut self, hosts: &mut HashMap<SocketAddr, HostStatus>) -> SocketAddr {
        for _host in &self.hosts_vec {
            self.last_host = (self.last_host + 1) % self.hosts_vec.len();
            if let Some(current_host) = hosts.get(&self.hosts_vec[self.last_host]) {
                if current_host.healthy {
                    return self.hosts_vec[self.last_host];
                }
            }
        }

        panic!("No healthy hosts");
    }
}
