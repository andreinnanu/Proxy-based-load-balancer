use std::{collections::HashMap, net::SocketAddr};

use crate::services::{Algorithm, HostStatus, Strategy};

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
    fn get_host(&mut self, hosts: &mut HashMap<SocketAddr, HostStatus>) -> Option<SocketAddr> {
        for _host in &self.hosts_vec {
            self.last_host = (self.last_host + 1) % self.hosts_vec.len();
            if let Some(current_host) = hosts.get(&self.hosts_vec[self.last_host]) {
                if current_host.healthy {
                    return Some(self.hosts_vec[self.last_host]);
                }
            }
        }

        None
    }
    
    fn get_strategy(&mut self) -> Strategy {
        Strategy::RoundRobin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::collections::HashMap;

    fn make_addr(octet: u8) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, octet)), 8080)
    }

    fn host(healthy: bool, open_connections: u32) -> HostStatus {
        HostStatus { healthy, open_connections }
    }

    #[fixture]
    fn healthy_hosts() -> HashMap<SocketAddr, HostStatus> {
        let mut hosts = HashMap::new();
        hosts.insert(make_addr(1), host(true, 0));
        hosts.insert(make_addr(2), host(true, 0));
        hosts.insert(make_addr(3), host(true, 0));
        hosts.insert(make_addr(4), host(true, 0));
        hosts
    }

    #[fixture]
    fn rr(mut healthy_hosts: HashMap<SocketAddr, HostStatus>) -> RoundRobin {
        RoundRobin::new(&mut healthy_hosts)
    }

    #[rstest]
    fn test_round_robin_cycles(mut rr: RoundRobin, mut healthy_hosts: HashMap<SocketAddr, HostStatus>) {
        let first_host = rr.get_host(&mut healthy_hosts);
        
        for _ in 0..healthy_hosts.keys().len() - 1 {
            let _ = rr.get_host(&mut healthy_hosts);
        }

        let first_after_cycle = rr.get_host(&mut healthy_hosts);
        assert_eq!(first_host, first_after_cycle);
    }

    #[rstest]
    fn test_round_robin_skips_unhealthy() {
        let mut hosts = HashMap::new();
        hosts.insert(make_addr(1), host(true, 5));
        hosts.insert(make_addr(2), host(false, 10));

        let mut rr = RoundRobin::new(&mut hosts);

        assert_eq!(rr.get_host(&mut hosts), Some(make_addr(1)));
        assert_eq!(rr.get_host(&mut hosts), Some(make_addr(1)));
    }

    #[rstest]
    fn test_round_robin_all_unhealthy() {
        let mut hosts = HashMap::new();
        hosts.insert(make_addr(1), host(false, 2));
        hosts.insert(make_addr(2), host(false, 7));

        let mut rr = RoundRobin::new(&mut hosts);

        assert_eq!(rr.get_host(&mut hosts), None);
    }

    #[rstest]
    fn test_round_robin_empty_hosts() {
        let mut hosts = HashMap::new();
        let mut rr = RoundRobin::new(&mut hosts);

        assert_eq!(rr.get_host(&mut hosts), None);
    }
}

