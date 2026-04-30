use anyhow::{Context, Result};
use serde_json::json;
use crate::lib::client::RelayClient;
use crate::lib::types::Execute;

pub async fn run(
    client: &RelayClient,
    from_chain: u64,
    from_currency: &str,
    to_chain: u64,
    to_currency: &str,
    amount: &str,
    user: &str,
    recipient: Option<&str>,
) -> Result<Execute> {
    let body = json!({
        "originChainId": from_chain,
        "originCurrency": from_currency,
        "destinationChainId": to_chain,
        "destinationCurrency": to_currency,
        "amount": amount,
        "user": user,
        "recipient": recipient.unwrap_or(user),
        "tradeType": "EXACT_INPUT",
    });

    let url = client.url("/quote");
    let resp = client.http.post(&url).json(&body).send().await
        .context("failed to reach Relay API — check your internet connection")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("quote failed (HTTP {}): {}", status, text.trim());
    }

    let quote: Execute = resp.json().await
        .context("unexpected response format from /quote")?;

    if let Some(errors) = &quote.errors {
        if !errors.is_empty() {
            let msg = errors.iter()
                .filter_map(|e| e.message.as_deref())
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!("quote error: {}", msg);
        }
    }

    Ok(quote)
}

pub fn print_quote(quote: &Execute) {
    if let Some(details) = &quote.details {
        if let Some(cin) = &details.currency_in {
            if let Some(currency) = &cin.currency {
                println!(
                    "  send:    {} {} ({})",
                    cin.amount_formatted.as_deref().unwrap_or("-"),
                    currency.symbol.as_deref().unwrap_or("?"),
                    currency.chain_id.map(|id| id.to_string()).as_deref().unwrap_or("?"),
                );
            }
        }
        if let Some(cout) = &details.currency_out {
            if let Some(currency) = &cout.currency {
                println!(
                    "  receive: {} {} ({})",
                    cout.amount_formatted.as_deref().unwrap_or("-"),
                    currency.symbol.as_deref().unwrap_or("?"),
                    currency.chain_id.map(|id| id.to_string()).as_deref().unwrap_or("?"),
                );
            }
        }
        if let Some(eta) = details.time_estimate {
            println!("  eta:     {:.1}s", eta);
        }
    }
}
