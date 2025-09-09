use std::{fs, net::SocketAddr, path::PathBuf};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub hosts: Vec<SocketAddr>
}

impl Config {
    pub fn new(config_file: &PathBuf) -> Self {
        let yaml = fs::read_to_string(config_file).expect("Failed to read configuration file");
        serde_yaml::from_str(&yaml).expect("Failed to deserialize configuration file")
    }
}