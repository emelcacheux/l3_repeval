use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct CommandConfig {
    pub name: String,
    pub cmd: String,
    pub regex: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub commands: Vec<CommandConfig>,
}

pub fn load(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let cfg: Config = toml::from_str(&content)?;
    Ok(cfg)
}
