use std::{net::SocketAddr, path::PathBuf};

use crate::{
    services::{Algorithm, RoundRobin}, utils::Config,
};

pub struct LoadBalancerState {
    hosts: Vec<SocketAddr>,
    algorithm: Box<dyn Algorithm>,
}

impl LoadBalancerState {
    pub fn new(config_file: &PathBuf) -> Self {
        let config = Config::new(config_file);

        Self {
            hosts: config.hosts,
            algorithm: Box::new(RoundRobin::default()),
        }
    }

    pub fn get_host(&mut self) -> SocketAddr {
        self.algorithm.get_host(&self.hosts)
    }
}
