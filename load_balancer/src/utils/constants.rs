pub mod algorithm {
    pub const SWITCH_ALGORITHM_ENDPOINT: &str = "/switch-algorithm";
    pub const STRATEGY_QUERY_PARAM: &str = "strategy";
}

pub mod health_check {
    pub const CHECK_INTERVAL_SEC: u64 = 2;
    pub const TIMEOUT_MS: u64 = 500;
}