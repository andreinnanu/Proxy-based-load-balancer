use std::{collections::HashMap, net::SocketAddr};

use crate::services::{Algorithm, HostStatus, Strategy};

#[derive(Clone, Debug, Default)]
pub struct LeastConnections;

impl Algorithm for LeastConnections {
    fn get_host(&mut self, hosts: &mut HashMap<SocketAddr, HostStatus>) -> Option<SocketAddr> {
        if let Some((host, x)) = hosts
            .iter()
            .filter(|&(_host, host_status)| host_status.healthy)
            .min_by_key(|(_host, status)| status.open_connections)
        {
            println!("Host: {} with {} connections", host, x.open_connections);
            Some(*host)
        } else {
            None
        }
    }
    
    fn get_strategy(&mut self) -> Strategy {
        Strategy::LeastConnections
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn make_addr(octet: u8) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, octet)), 8080)
    }

    fn host(healthy: bool, open_connections: u32) -> HostStatus {
        HostStatus { healthy, open_connections }
    }

    #[rstest]
    fn test_choose_host_with_least_connections() {
        let mut hosts = HashMap::new();
        hosts.insert(make_addr(1), host(true, 5));
        hosts.insert(make_addr(2), host(true, 2));
        hosts.insert(make_addr(3), host(true, 7));

        let mut lb = LeastConnections;
        let chosen = lb.get_host(&mut hosts);

        assert_eq!(chosen, Some(make_addr(2)));
    }

    #[rstest]
    fn test_skip_unhealthy_hosts() {
        let mut hosts = HashMap::new();
        hosts.insert(make_addr(1), host(false, 0));
        hosts.insert(make_addr(2), host(true, 10));

        let mut lb = LeastConnections;
        let chosen = lb.get_host(&mut hosts);

        assert_eq!(chosen, Some(make_addr(2)));
    }

    #[rstest]
    fn test_all_unhealthy_returns_none() {
        let mut hosts = HashMap::new();
        hosts.insert(make_addr(1), host(false, 1));
        hosts.insert(make_addr(2), host(false, 5));

        let mut lb = LeastConnections;
        let chosen = lb.get_host(&mut hosts);

        assert_eq!(chosen, None);
    }

    #[rstest]
    fn test_empty_hosts_returns_none() {
        let mut hosts = HashMap::new();

        let mut lb = LeastConnections;
        let chosen = lb.get_host(&mut hosts);

        assert_eq!(chosen, None);
    }

    #[rstest]
    fn test_tie_returns_one_of_min_hosts() {
        let mut hosts = HashMap::new();
        let h1 = make_addr(1);
        let h2 = make_addr(2);

        hosts.insert(h1, host(true, 3));
        hosts.insert(h2, host(true, 3));

        let mut lb = LeastConnections;
        let chosen = lb.get_host(&mut hosts);

        assert!(chosen == Some(h1) || chosen == Some(h2));
    }
}