pub mod algorithm {
    pub const SWITCH_ALGORITHM_ENDPOINT: &str = "/switch-algorithm";
    pub const STRATEGY_QUERY_PARAM: &str = "strategy";
    pub const OVERLOAD_FACTOR_CONNECTIONS: f64 = 10.0;
    pub const OVERLOAD_FACTOR_LATENCY: f64 = 3.0;
    pub const ALGORITHM_SWITCH_TIMEOUT_SEC: u64 = 30;
}

pub mod health_check {
    pub const CHECK_INTERVAL_SEC: u64 = 5;
    pub const TIMEOUT_MS: u64 = 1000;
}
