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
    }
    Ok(())
}
