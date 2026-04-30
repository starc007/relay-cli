use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    pub api_key: Option<String>,
    pub private_key: Option<String>,
    pub testnet: Option<bool>,
    #[serde(default)]
    pub rpcs: HashMap<u64, String>,
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".relay")
        .join("config.json")
}

fn default_rpcs() -> HashMap<u64, String> {
    [
        (1,     "https://ethereum.publicnode.com"),
        (8453,  "https://base.publicnode.com"),
        (42161, "https://arbitrum-one.publicnode.com"),
        (10,    "https://optimism.publicnode.com"),
        (137,   "https://polygon-bor.publicnode.com"),
        (43114, "https://avalanche-c-chain.publicnode.com"),
        (56,    "https://bsc.publicnode.com"),
        (324,   "https://mainnet.era.zksync.io"),
        (534352,"https://rpc.scroll.io"),
        (59144, "https://rpc.linea.build"),
        (7777777,"https://rpc.zora.energy"),
    ]
    .into_iter()
    .map(|(id, url)| (id, url.to_string()))
    .collect()
}

pub fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        let cfg = Config {
            rpcs: default_rpcs(),
            ..Default::default()
        };
        save(&cfg)?;
        return Ok(cfg);
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut cfg: Config = serde_json::from_str(&raw).context("failed to parse config")?;
    // Merge defaults for any chain not already set
    for (id, url) in default_rpcs() {
        cfg.rpcs.entry(id).or_insert(url);
    }
    Ok(cfg)
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, raw)
        .with_context(|| format!("failed to write {}", path.display()))
}

pub fn rpc_for_chain(config: &Config, chain_id: u64) -> Option<String> {
    // Env var takes priority
    if let Ok(val) = std::env::var(format!("RPC_{}", chain_id)) {
        return Some(val);
    }
    config.rpcs.get(&chain_id).cloned()
}
