use std::net::SocketAddr;

pub enum Strategy {
    RoundRobin
}

pub trait Algorithm: Send + Sync {
    fn get_host(&mut self, hosts: &[SocketAddr]) -> SocketAddr;
}

#[derive(Clone, Debug)]
pub struct RoundRobin {
    last_host: usize
}

impl RoundRobin {
    pub fn new() -> Self {
        Self { last_host: 0 }
    }
}

impl Algorithm for RoundRobin {
    fn get_host(&mut self, hosts: &[SocketAddr]) -> SocketAddr {
        self.last_host = (self.last_host + 1) % hosts.len();

        hosts[self.last_host]
    }
}