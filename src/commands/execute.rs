use anyhow::{bail, Context, Result};
use alloy::{
    dyn_abi::TypedData,
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::{local::PrivateKeySigner, Signer},
};
use indicatif::{ProgressBar, ProgressStyle};
use crate::lib::{client::RelayClient, config::Config, types::{Execute, StepKind}};

pub async fn run(client: &RelayClient, cfg: &Config, quote: Execute, signer: PrivateKeySigner) -> Result<()> {
    let wallet = EthereumWallet::from(signer.clone());

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
                    let rpc_url = rpc_url_for_chain(cfg, chain_id)?;

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
                    let sign = data.sign.as_ref().context("signature step missing sign data")?;
                    let post = data.post.as_ref().context("signature step missing post data")?;

                    let sig_kind = sign.signature_kind.as_deref().unwrap_or("eip191");
                    let signature = match sig_kind {
                        "eip712" => {
                            let typed_data: TypedData = serde_json::from_value(serde_json::json!({
                                "domain": sign.domain,
                                "types": sign.types,
                                "primaryType": sign.primary_type,
                                "message": sign.value,
                            }))
                            .context("failed to parse eip712 typed data")?;
                            let hash = typed_data
                                .eip712_signing_hash()
                                .context("failed to compute eip712 hash")?;
                            signer.sign_hash(&hash).await?.to_string()
                        }
                        _ => {
                            let message = sign.message.as_deref().unwrap_or_default();
                            signer.sign_message(message.as_bytes()).await?.to_string()
                        }
                    };

                    let endpoint = post.endpoint.as_deref().context("post missing endpoint")?;
                    let mut body = post.body.clone().unwrap_or(serde_json::Value::Object(Default::default()));
                    if let serde_json::Value::Object(ref mut map) = body {
                        map.insert("signature".to_string(), serde_json::Value::String(signature.clone()));
                    }

                    let method = post.method.as_deref().unwrap_or("POST");
                    let url = client.url(endpoint);
                    let req = match method.to_uppercase().as_str() {
                        "GET" => client.http.get(&url),
                        _ => client.http.post(&url),
                    };
                    req.json(&body).send().await?;

                    spinner.set_message(format!("{} — signed: {}…", step.action, &signature[..10]));
                }
            }
        }

        spinner.finish_with_message(format!("{} ✓", step.action));
    }

    println!("\nbridge complete");
    Ok(())
}

fn rpc_url_for_chain(cfg: &Config, chain_id: u64) -> Result<String> {
    crate::lib::config::rpc_for_chain(cfg, chain_id)
        .ok_or_else(|| anyhow::anyhow!(
            "no RPC for chain {}. Run: relay config set-rpc --chain {} --url <url>",
            chain_id, chain_id
        ))
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
