use anyhow::{Context, Result, bail};
use alloy::{
    dyn_abi::TypedData,
    network::{EthereumWallet, TransactionBuilder},
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::{local::PrivateKeySigner, Signer},
};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use crate::lib::{client::RelayClient, config::Config, types::{Execute, StepKind}};

const MAX_TX_RETRIES: u32 = 2;
const RETRY_DELAY_MS: u64 = 2000;
const STATUS_POLL_MS: u64 = 3000;
const STATUS_TIMEOUT_SECS: u64 = 120;

pub async fn run(client: &RelayClient, cfg: &Config, quote: Execute, signer: PrivateKeySigner) -> Result<()> {
    let wallet = EthereumWallet::from(signer.clone());

    // collect request_id for post-execution status polling
    let request_id = quote.steps.iter().find_map(|s| s.request_id.clone());

    for step in &quote.steps {
        let spinner = new_spinner();
        spinner.set_message(step.action.clone());

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

                    let base_provider = ProviderBuilder::new()
                        .on_builtin(&rpc_url)
                        .await
                        .with_context(|| format!("failed to connect to RPC for chain {chain_id}: {rpc_url}"))?;

                    let wallet_provider = ProviderBuilder::new()
                        .wallet(wallet.clone())
                        .on_builtin(&rpc_url)
                        .await?;

                    let to: Address = data
                        .to
                        .as_deref()
                        .unwrap_or_default()
                        .parse()
                        .context("invalid destination address in step data")?;

                    let value = data
                        .value
                        .as_deref()
                        .unwrap_or("0")
                        .parse::<U256>()
                        .unwrap_or_default();

                    let calldata: alloy::primitives::Bytes = data
                        .data
                        .as_ref()
                        .and_then(|d| d.as_str())
                        .unwrap_or("0x")
                        .parse()
                        .unwrap_or_default();

                    let max_fee_per_gas: u128 = data
                        .max_fee_per_gas
                        .as_deref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    let max_priority_fee_per_gas: u128 = data
                        .max_priority_fee_per_gas
                        .as_deref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    let gas_limit: u64 = data
                        .gas
                        .as_deref()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(300_000);

                    // retry loop — re-fetch nonce each attempt to handle conflicts
                    let mut last_err = None;
                    let mut tx_hash = None;

                    for attempt in 0..=MAX_TX_RETRIES {
                        if attempt > 0 {
                            spinner.set_message(format!(
                                "{} — retry {}/{}",
                                step.action, attempt, MAX_TX_RETRIES
                            ));
                            tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                        }

                        let nonce = base_provider
                            .get_transaction_count(signer.address())
                            .await
                            .context("failed to fetch nonce")?;

                        let mut tx = TransactionRequest::default()
                            .to(to)
                            .value(value)
                            .input(calldata.clone().into())
                            .nonce(nonce)
                            .max_fee_per_gas(max_fee_per_gas)
                            .max_priority_fee_per_gas(max_priority_fee_per_gas)
                            .gas_limit(gas_limit);
                        tx.set_chain_id(chain_id);

                        match wallet_provider.send_transaction(tx).await {
                            Ok(pending) => {
                                tx_hash = Some(*pending.tx_hash());
                                break;
                            }
                            Err(e) => {
                                let msg = e.to_string();
                                // surface actionable errors immediately
                                if msg.contains("insufficient funds") {
                                    bail!("insufficient funds — wallet doesn't have enough ETH to cover amount + gas");
                                }
                                if msg.contains("rejected") || msg.contains("denied") {
                                    bail!("transaction rejected by RPC: {}", msg);
                                }
                                last_err = Some(e);
                            }
                        }
                    }

                    let hash = tx_hash.ok_or_else(|| {
                        let e = last_err.unwrap();
                        anyhow::anyhow!("transaction failed after {} retries: {}", MAX_TX_RETRIES, e)
                    })?;

                    spinner.set_message(format!("{} — tx: {}", step.action, hash));
                    post_step_check(client, &step.id, &item.check).await?;
                }

                StepKind::Signature => {
                    let sign = data.sign.as_ref().context("signature step missing sign field")?;
                    let post = data.post.as_ref().context("signature step missing post field")?;

                    let sig_kind = sign.signature_kind.as_deref().unwrap_or("eip191");
                    let signature = match sig_kind {
                        "eip712" => {
                            let typed_data: TypedData = serde_json::from_value(serde_json::json!({
                                "domain": sign.domain,
                                "types": sign.types,
                                "primaryType": sign.primary_type,
                                "message": sign.value,
                            }))
                            .context("failed to parse EIP-712 typed data from step")?;
                            let hash = typed_data
                                .eip712_signing_hash()
                                .context("failed to compute EIP-712 signing hash")?;
                            signer.sign_hash(&hash).await?.to_string()
                        }
                        _ => {
                            let message = sign.message.as_deref().unwrap_or_default();
                            signer.sign_message(message.as_bytes()).await?.to_string()
                        }
                    };

                    let endpoint = post.endpoint.as_deref().context("signature post step missing endpoint")?;
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
                    req.json(&body).send().await
                        .context("failed to post signature to Relay API")?;

                    spinner.set_message(format!("{} — signed", step.action));
                }
            }
        }

        spinner.finish_with_message(format!("{} ✓", step.action));
    }

    // Poll until bridge confirms or times out
    if let Some(rid) = request_id {
        poll_bridge_status(client, &rid).await?;
    } else {
        println!("\nbridge submitted — no request ID to track");
    }

    Ok(())
}

async fn poll_bridge_status(client: &RelayClient, request_id: &str) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct StatusResp {
        status: Option<String>,
        details: Option<String>,
    }

    let spinner = new_spinner();
    spinner.set_message("waiting for bridge confirmation...");

    let url = client.url(&format!("/intents/status/v2?requestId={}", request_id));
    let deadline = tokio::time::Instant::now() + Duration::from_secs(STATUS_TIMEOUT_SECS);

    loop {
        let resp: StatusResp = client.http.get(&url).send().await
            .context("failed to poll bridge status")?
            .json().await
            .context("failed to parse bridge status response")?;

        let status = resp.status.as_deref().unwrap_or("unknown");

        match status {
            "success" => {
                spinner.finish_with_message("bridge confirmed ✓");
                return Ok(());
            }
            "failure" | "refund" => {
                spinner.finish_and_clear();
                let detail = resp.details.as_deref().unwrap_or("no details");
                bail!("bridge {}: {}", status, detail);
            }
            _ => {
                spinner.set_message(format!("status: {} — waiting...", status));
            }
        }

        if tokio::time::Instant::now() >= deadline {
            spinner.finish_and_clear();
            bail!(
                "bridge timed out after {}s — check status manually:\n  relay status {}",
                STATUS_TIMEOUT_SECS, request_id
            );
        }

        tokio::time::sleep(Duration::from_millis(STATUS_POLL_MS)).await;
    }
}

fn new_spinner() -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

fn rpc_url_for_chain(cfg: &Config, chain_id: u64) -> Result<String> {
    crate::lib::config::rpc_for_chain(cfg, chain_id)
        .ok_or_else(|| anyhow::anyhow!(
            "no RPC configured for chain {chain_id}\n  fix: relay config set-rpc --chain {chain_id} --url <rpc-url>"
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
    client.http.get(&url).send().await
        .context("failed to call step check endpoint")?;
    Ok(())
}
