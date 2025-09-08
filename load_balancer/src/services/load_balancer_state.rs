use std::net::SocketAddr;

use crate::services::{Algorithm, RoundRobin};

pub struct LoadBalancerState {
    hosts: Vec<SocketAddr>,
    algorithm: Box<dyn Algorithm>,
}

impl LoadBalancerState {
    pub fn new() -> Self {
        Self {
            hosts: vec![
                "0.0.0.0:8081".parse().unwrap(),
                "0.0.0.0:8082".parse().unwrap(),
                "0.0.0.0:8083".parse().unwrap(),
                "0.0.0.0:8084".parse().unwrap(),
            ],
            algorithm: Box::new(RoundRobin::default()),
        }
    }

    pub fn get_host(&mut self) -> SocketAddr {
        self.algorithm.get_host(&self.hosts)
    }
}
