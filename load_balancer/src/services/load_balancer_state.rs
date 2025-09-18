use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

use crate::{
    services::{Algorithm, LeastConnections, RoundRobin, Strategy},
    utils::Config,
};

#[derive(Default)]
pub struct HostStatus {
    pub healthy: bool,
    pub open_connections: u32,
}

impl HostStatus {
    pub fn new() -> Self {
        Self {
            healthy: true,
            open_connections: 0,
        }
    }
}

pub struct LoadBalancerState {
    pub hosts: HashMap<SocketAddr, HostStatus>,
    pub algorithm: Box<dyn Algorithm>,
}

impl LoadBalancerState {
    pub fn new(config_file: &PathBuf) -> Self {
        let config = Config::new(config_file);

        let mut hosts = HashMap::new();

        for host in config.hosts {
            hosts.insert(host, HostStatus::new());
        }

        let algorithm = Box::new(RoundRobin::new(&mut hosts));
        
        Self { hosts, algorithm }
    }

    pub fn get_host(&mut self) -> Option<SocketAddr> {
        if let Some(host) = self.algorithm.get_host(&mut self.hosts) {
            self.increase_connections(&host);
            return Some(host);
        }
        
        None
    }

    pub fn on_disconnect(&mut self, host: &SocketAddr) {
        self.decrease_connections(host);
    }

    pub fn set_algorithm(&mut self, strategy: Strategy) {
        self.algorithm = match strategy {
            Strategy::RoundRobin => Box::new(RoundRobin::new(&mut self.hosts)),
            Strategy::LeastConnections => Box::new(LeastConnections),
        }
    }

    pub fn set_host_health(&mut self, host: &SocketAddr, health: bool) {
        if let Some(host_status) = self.hosts.get_mut(host) {
            host_status.healthy = health;
        }
    }

    fn increase_connections(&mut self, host: &SocketAddr) {
        if let Some(status) = self.hosts.get_mut(host) {
            status.open_connections += 1;
        }
    }

    fn decrease_connections(&mut self, host: &SocketAddr) {
        if let Some(status) = self.hosts.get_mut(host) {
            status.open_connections -= 1;
        }
    }
}
