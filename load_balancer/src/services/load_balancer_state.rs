use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

use statrs::statistics::{Data, OrderStatistics};

use crate::{
    services::{Algorithm, LeastConnections, RoundRobin, Strategy},
    utils::{Config, algorithm::OVERLOAD_FACTOR},
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

    pub fn set_algorithm(&mut self, strategy: Strategy) -> bool {
        if self.algorithm.get_strategy() != strategy {
            println!("Switching to {}", strategy.as_ref());
            self.algorithm = match strategy {
                Strategy::RoundRobin => Box::new(RoundRobin::new(&mut self.hosts)),
                Strategy::LeastConnections => Box::new(LeastConnections),
            };

            return true;
        }

        false
    }

    pub fn set_host_health(&mut self, host: &SocketAddr, health: bool) {
        if let Some(host_status) = self.hosts.get_mut(host) {
            host_status.healthy = health;
        }
    }

    pub fn is_worker_overloaded(&self) -> bool {
        let connections: Vec<f64> = self
            .hosts
            .values()
            .map(|host_status| host_status.open_connections as f64)
            .collect();

        let mut dataset = Data::new(connections.clone());
        let median = dataset.median();

        let deviations: Vec<f64> = connections.iter().map(|x| (x - median).abs()).collect();

        let mad = Data::new(deviations).median().max(1.0);

        connections
            .iter()
            .any(|&x| (x - median).abs() > OVERLOAD_FACTOR * mad)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn make_addr(octet: u8) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, octet)), 8080)
    }

    fn make_lb_with_round_robin() -> LoadBalancerState {
        let mut hosts = HashMap::new();
        hosts.insert(make_addr(1), HostStatus::new());
        hosts.insert(make_addr(2), HostStatus::new());
        hosts.insert(make_addr(3), HostStatus::new());

        let algorithm = Box::new(RoundRobin::new(&mut hosts));
        LoadBalancerState { hosts, algorithm }
    }

    fn make_lb_with_least_connections() -> LoadBalancerState {
        let mut hosts = HashMap::new();
        hosts.insert(make_addr(1), HostStatus::new());
        hosts.insert(make_addr(2), HostStatus::new());
        hosts.insert(make_addr(3), HostStatus::new());

        let algorithm = Box::new(LeastConnections);
        LoadBalancerState { hosts, algorithm }
    }

    #[rstest]
    fn test_round_robin_get_host_increases_connections() {
        let mut lb = make_lb_with_round_robin();

        let h1 = lb.get_host().unwrap();
        assert_eq!(lb.hosts[&h1].open_connections, 1);

        let h2 = lb.get_host().unwrap();
        assert_eq!(lb.hosts[&h2].open_connections, 1);

        let h3 = lb.get_host().unwrap();
        assert_eq!(lb.hosts[&h3].open_connections, 1);
    }

    #[rstest]
    fn test_on_disconnect_decreases_connections() {
        let mut lb = make_lb_with_round_robin();

        let h1 = lb.get_host().unwrap();
        assert_eq!(lb.hosts[&h1].open_connections, 1);

        lb.on_disconnect(&h1);
        assert_eq!(lb.hosts[&h1].open_connections, 0);
    }

    #[rstest]
    fn test_set_algorithm_switches_strategy() {
        let mut lb = make_lb_with_round_robin();
        lb.set_algorithm(Strategy::LeastConnections);

        assert!(lb.get_host().is_some());
    }

    #[rstest]
    fn test_set_host_health_skips_unhealthy() {
        let mut lb = make_lb_with_least_connections();
        let h1 = make_addr(1);

        lb.set_host_health(&h1, false);

        for _ in 0..5 {
            let chosen = lb.get_host().unwrap();
            assert_ne!(chosen, h1);
        }
    }

    #[rstest]
    fn test_get_host_returns_none_if_all_unhealthy() {
        let mut lb = make_lb_with_round_robin();
        for host in lb.hosts.keys().cloned().collect::<Vec<_>>() {
            lb.set_host_health(&host, false);
        }

        assert_eq!(lb.get_host(), None);
    }

    #[rstest]
    fn test_is_worker_overloaded_detects_outlier() {
        let mut lb = make_lb_with_round_robin();

        for (_, status) in lb.hosts.iter_mut() {
            status.open_connections = 10;
        }
        assert!(!lb.is_worker_overloaded());

        let first_host = *lb.hosts.keys().next().unwrap();
        lb.hosts.get_mut(&first_host).unwrap().open_connections = 1000;

        assert!(lb.is_worker_overloaded());
    }
}
