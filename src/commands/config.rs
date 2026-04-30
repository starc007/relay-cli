use anyhow::Result;
use clap::Subcommand;
use crate::lib::config;

#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    Show,
    Set {
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        testnet: Option<bool>,
    },
}

pub async fn run(cmd: ConfigCmd) -> Result<()> {
    match cmd {
        ConfigCmd::Show => {
            let cfg = config::load()?;
            println!("{}", serde_json::to_string_pretty(&cfg)?);
        }
        ConfigCmd::Set { api_key, testnet } => {
            let mut cfg = config::load()?;
            if let Some(k) = api_key {
                cfg.api_key = Some(k);
            }
            if let Some(t) = testnet {
                cfg.testnet = Some(t);
            }
            config::save(&cfg)?;
            println!("config saved");
        }
    }
    Ok(())
}
