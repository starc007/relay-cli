use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod lib;

#[derive(Debug, Parser)]
#[command(name = "relay", about = "Cross-chain bridge/swap via Relay protocol", version)]
struct Cli {
    #[arg(long, env = "RELAY_API_KEY", global = true)]
    api_key: Option<String>,

    #[arg(long, global = true)]
    testnet: bool,

    #[arg(long, env = "RELAY_PRIVATE_KEY", global = true, hide_env_values = true)]
    private_key: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Chains {
        #[arg(long, short)]
        filter: Option<String>,
    },
    Quote {
        #[arg(long, value_name = "CHAIN_ID")]
        from_chain: u64,
        #[arg(long, value_name = "ADDRESS_OR_SYMBOL")]
        from_currency: String,
        #[arg(long, value_name = "CHAIN_ID")]
        to_chain: u64,
        #[arg(long, value_name = "ADDRESS_OR_SYMBOL")]
        to_currency: String,
        #[arg(long)]
        amount: String,
        #[arg(long, env = "RELAY_WALLET")]
        user: String,
        #[arg(long)]
        recipient: Option<String>,
    },
    Bridge {
        #[arg(long, value_name = "CHAIN_ID")]
        from_chain: u64,
        #[arg(long, value_name = "ADDRESS_OR_SYMBOL")]
        from_currency: String,
        #[arg(long, value_name = "CHAIN_ID")]
        to_chain: u64,
        #[arg(long, value_name = "ADDRESS_OR_SYMBOL")]
        to_currency: String,
        #[arg(long)]
        amount: String,
        #[arg(long, env = "RELAY_WALLET")]
        user: String,
        #[arg(long)]
        recipient: Option<String>,
    },
    Tokens {
        #[arg(long, value_name = "CHAIN_ID")]
        chain: u64,
        #[arg(long, short)]
        filter: Option<String>,
        #[arg(long, help = "show verified tokens only")]
        verified: bool,
    },
    Status {
        request_id: String,
        #[arg(long, short, help = "poll until terminal state")]
        watch: bool,
    },
    Config {
        #[command(subcommand)]
        cmd: commands::config::ConfigCmd,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let cfg = lib::config::load()?;
    let api_key = cli.api_key.or(cfg.api_key);
    let testnet = cli.testnet || cfg.testnet.unwrap_or(false);

    let client = lib::client::RelayClient::new(api_key.as_deref(), testnet)?;

    match cli.command {
        Command::Chains { filter } => {
            commands::chains::run(&client, filter.as_deref()).await?;
        }
        Command::Quote {
            from_chain,
            from_currency,
            to_chain,
            to_currency,
            amount,
            user,
            recipient,
        } => {
            let quote = commands::quote::run(
                &client,
                from_chain,
                &from_currency,
                to_chain,
                &to_currency,
                &amount,
                &user,
                recipient.as_deref(),
            )
            .await?;
            commands::quote::print_quote(&quote);
        }
        Command::Bridge {
            from_chain,
            from_currency,
            to_chain,
            to_currency,
            amount,
            user,
            recipient,
        } => {
            let signer = lib::wallet::load_signer(cli.private_key.as_deref())?;
            let quote = commands::quote::run(
                &client,
                from_chain,
                &from_currency,
                to_chain,
                &to_currency,
                &amount,
                &user,
                recipient.as_deref(),
            )
            .await?;
            println!("quote:");
            commands::quote::print_quote(&quote);
            println!("\nexecuting...");
            commands::execute::run(&client, quote, signer).await?;
        }
        Command::Tokens { chain, filter, verified } => {
            commands::tokens::run(&client, chain, filter.as_deref(), verified).await?;
        }
        Command::Status { request_id, watch } => {
            commands::status::run(&client, &request_id, watch).await?;
        }
        Command::Config { cmd } => {
            commands::config::run(cmd).await?;
        }
    }

    Ok(())
}
