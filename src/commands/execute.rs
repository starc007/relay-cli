use anyhow::{bail, Result};
use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use indicatif::{ProgressBar, ProgressStyle};
use crate::lib::{client::RelayClient, types::{Execute, StepKind}};

pub async fn run(client: &RelayClient, quote: Execute, signer: PrivateKeySigner) -> Result<()> {
    let wallet = EthereumWallet::from(signer);

    for step in &quote.steps {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        spinner.set_message(format!("{}", step.action));
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        for item in &step.items {
            if item.status == "complete" {
                continue;
            }

            let Some(data) = &item.data else {
                continue;
            };

            match step.kind {
                StepKind::Transaction => {
                    let chain_id = data.chain_id.unwrap_or(1);
                    let rpc_url = rpc_url_for_chain(chain_id)?;

                    let provider = ProviderBuilder::new()
                        .wallet(wallet.clone())
                        .on_builtin(&rpc_url)
                        .await?;

                    let to: Address = data
                        .to
                        .as_deref()
                        .unwrap_or_default()
                        .parse()
                        .unwrap_or_default();

                    let value = data
                        .value
                        .as_deref()
                        .unwrap_or("0")
                        .parse::<U256>()
                        .unwrap_or_default();

                    let calldata = data
                        .data
                        .as_ref()
                        .and_then(|d| d.as_str())
                        .unwrap_or("0x")
                        .to_string();

                    let tx = TransactionRequest::default()
                        .to(to)
                        .value(value)
                        .input(calldata.parse::<alloy::primitives::Bytes>().unwrap_or_default().into());

                    let tx_hash = *provider.send_transaction(tx).await?.tx_hash();

                    spinner.set_message(format!(
                        "{} — tx: {}",
                        step.action,
                        tx_hash
                    ));

                    post_step_check(client, &step.id, &item.check).await?;
                }
                StepKind::Signature => {
                    bail!("signature steps not yet implemented");
                }
            }
        }

        spinner.finish_with_message(format!("{} ✓", step.action));
    }

    println!("\nbridge complete");
    Ok(())
}

fn rpc_url_for_chain(chain_id: u64) -> Result<String> {
    if let Ok(val) = std::env::var(format!("RPC_{}", chain_id)) {
        return Ok(val);
    }
    bail!(
        "no RPC URL for chain {}. Set RPC_{} env var.",
        chain_id,
        chain_id
    )
}

async fn post_step_check(
    client: &RelayClient,
    _step_id: &Option<String>,
    check: &Option<crate::lib::types::CheckApi>,
) -> Result<()> {
    let Some(check) = check else { return Ok(()) };
    let Some(endpoint) = &check.endpoint else { return Ok(()) };
    let url = client.url(endpoint);
    let _ = client.http.get(&url).send().await?;
    Ok(())
}
