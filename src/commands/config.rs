use anyhow::Result;
use clap::Subcommand;
use crate::lib::config;

#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    Show,
    Set {
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long, hide_env_values = true)]
        private_key: Option<String>,
        #[arg(long)]
        testnet: Option<bool>,
    },
    SetRpc {
        #[arg(long, value_name = "CHAIN_ID")]
        chain: u64,
        #[arg(long, value_name = "URL")]
        url: String,
    },
    ListRpcs,
}

pub async fn run(cmd: ConfigCmd) -> Result<()> {
    match cmd {
        ConfigCmd::Show => {
            let cfg = config::load()?;
            let mut display = cfg.clone();
            if display.private_key.is_some() {
                display.private_key = Some("***".to_string());
            }
            println!("{}", serde_json::to_string_pretty(&display)?);
        }
        ConfigCmd::Set { api_key, private_key, testnet } => {
            let mut cfg = config::load()?;
            if let Some(k) = api_key {
                cfg.api_key = Some(k);
            }
            if let Some(k) = private_key {
                cfg.private_key = Some(k);
            }
            if let Some(t) = testnet {
                cfg.testnet = Some(t);
            }
            config::save(&cfg)?;
            println!("config saved");
        }
        ConfigCmd::SetRpc { chain, url } => {
            let mut cfg = config::load()?;
            cfg.rpcs.insert(chain, url.clone());
            config::save(&cfg)?;
            println!("rpc for chain {} set to {}", chain, url);
        }
        ConfigCmd::ListRpcs => {
            let cfg = config::load()?;
            let mut chains: Vec<_> = cfg.rpcs.iter().collect();
            chains.sort_by_key(|(id, _)| *id);
            println!("{:<12} {}", "CHAIN ID", "RPC URL");
            println!("{}", "-".repeat(70));
            for (id, url) in chains {
                println!("{:<12} {}", id, url);
            }
        }
    }
    Ok(())
}
