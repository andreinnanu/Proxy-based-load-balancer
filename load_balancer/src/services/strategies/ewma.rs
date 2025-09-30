use std::{collections::HashMap, net::SocketAddr};

use crate::services::{Algorithm, HostStatus, Strategy};

#[derive(Clone, Debug, Default)]
pub struct EWMA {
    pub scores: HashMap<SocketAddr, f64>,
}

const INITIAL_EWMA_SCORE: f64 = 100.0;
const ALPHA: f64 = 0.5;

impl EWMA {
    pub fn new(hosts: &mut HashMap<SocketAddr, HostStatus>) -> Self {
        let scores: HashMap<SocketAddr, f64> =
            hosts.keys().map(|k| (*k, INITIAL_EWMA_SCORE)).collect();

        Self { scores }
    }

    fn calculate_new_ewma_scores(&mut self, hosts: &mut HashMap<SocketAddr, HostStatus>) {
        for (host, status) in hosts {
            if let Some(ewma) = self.scores.get_mut(host) {
                let latency_ms = status.last_request_latency.as_secs_f64() * 1000.0;
                *ewma = status.open_connections as f64 + ALPHA * latency_ms + (1.0 - ALPHA) * *ewma;
            } else {
                unreachable!("Keys should always match");
            }
        }
    }
}

impl Algorithm for EWMA {
    fn get_host(&mut self, hosts: &mut HashMap<SocketAddr, HostStatus>) -> Option<SocketAddr> {
        self.calculate_new_ewma_scores(hosts);

        let mut min = 10000000.0;
        let mut min_host = None;

        for (host, _) in hosts
            .iter()
            .filter(|&(_host, host_status)| host_status.healthy)
        {
            if let Some(score) = self.scores.get(host) {
                if *score < min {
                    min = *score;
                    min_host = Some(*host);
                }
            }
        }

        min_host
    }

    fn get_strategy(&self) -> Strategy {
        Strategy::EWMA
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;

    fn make_addr(octet: u8) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, octet)), 8080)
    }

    fn make_hosts(latencies: &[u64]) -> HashMap<SocketAddr, HostStatus> {
        let mut hosts = HashMap::new();
        for (i, &latency) in latencies.iter().enumerate() {
            hosts.insert(
                make_addr((i + 1) as u8),
                HostStatus {
                    healthy: true,
                    open_connections: 0,
                    last_request_latency: Duration::from_millis(latency),
                },
            );
        }
        hosts
    }

    #[rstest]
    fn ewma_initialization() {
        let mut hosts = make_hosts(&[100, 200, 50]);
        let ewma = EWMA::new(&mut hosts);

        assert_eq!(ewma.scores.len(), hosts.len());
        for score in ewma.scores.values() {
            assert_eq!(*score, INITIAL_EWMA_SCORE);
        }
    }

    #[rstest]
    fn ewma_score_update() {
        let mut hosts = make_hosts(&[100, 200, 50]);
        let mut ewma = EWMA::new(&mut hosts);

        ewma.calculate_new_ewma_scores(&mut hosts);

        for (host, status) in hosts.iter() {
            let expected = ALPHA * status.last_request_latency.as_secs_f64() * 1000.0
                + (1.0 - ALPHA) * INITIAL_EWMA_SCORE;
            let score = ewma.scores.get(host).unwrap();
            assert!((score - expected).abs() < 1e-6);
        }
    }

    #[rstest]
    fn ewma_get_host() {
        let mut hosts = make_hosts(&[100, 200, 50]);
        let mut ewma = EWMA::new(&mut hosts);

        ewma.calculate_new_ewma_scores(&mut hosts);
        let selected = ewma.get_host(&mut hosts).unwrap();

        assert_eq!(selected, make_addr(3));
    }

    #[rstest]
    fn ewma_strategy() {
        let mut hosts = make_hosts(&[100, 200, 50]);
        let ewma = EWMA::new(&mut hosts);

        assert_eq!(ewma.get_strategy(), Strategy::EWMA);
    }

    #[rstest]
    #[case(&[100, 200, 50], 5)]
    #[case(&[50, 50, 50], 3)]
    #[case(&[300, 100, 200], 10)]
    fn ewma_multiple_rounds(#[case] latencies: &[u64], #[case] rounds: usize) {
        let mut hosts = make_hosts(latencies);
        let mut ewma = EWMA::new(&mut hosts);

        let mut last_selected = None;

        for _ in 0..rounds {
            ewma.calculate_new_ewma_scores(&mut hosts);

            let selected = ewma.get_host(&mut hosts).unwrap();

            last_selected = Some(selected);

            for (i, host_status) in hosts.values_mut().enumerate() {
                host_status.last_request_latency += Duration::from_millis(i as u64 * 10);
            }
        }

        for score in ewma.scores.values() {
            assert!(score.is_finite() && *score > 0.0);
        }

        assert!(hosts.contains_key(&last_selected.unwrap()));
    }
}
