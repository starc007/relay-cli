use anyhow::{bail, Result};
use serde_json::json;
use crate::lib::client::RelayClient;

const NATIVE_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
const NATIVE_DECIMALS: u8 = 18;

pub struct TokenInfo {
    pub address: String,
    pub decimals: u8,
}

pub async fn resolve(client: &RelayClient, input: &str, chain_id: u64) -> Result<TokenInfo> {
    // Already an address — fetch decimals from API
    if input.starts_with("0x") && input.len() == 42 {
        let decimals = fetch_decimals(client, input, chain_id).await.unwrap_or(18);
        return Ok(TokenInfo { address: input.to_string(), decimals });
    }

    // Native token shorthands
    let upper = input.to_uppercase();
    if matches!(upper.as_str(), "ETH" | "MATIC" | "BNB" | "AVAX" | "FTM" | "ONE" | "CELO" | "METIS") {
        return Ok(TokenInfo { address: NATIVE_ADDRESS.to_string(), decimals: NATIVE_DECIMALS });
    }

    // Lookup by symbol
    let body = json!({
        "chainIds": [chain_id],
        "term": input,
        "verified": true,
        "limit": 5,
    });

    let url = client.url("/currencies/v1");
    let results: Vec<Vec<serde_json::Value>> = client
        .http
        .post(&url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    let flat: Vec<&serde_json::Value> = results.iter().flatten().collect();
    let matched = flat.iter().find(|t| {
        t["symbol"]
            .as_str()
            .map(|s| s.eq_ignore_ascii_case(input))
            .unwrap_or(false)
    });

    if let Some(token) = matched {
        let address = token["address"].as_str()
            .ok_or_else(|| anyhow::anyhow!("token missing address"))?
            .to_string();
        let decimals = token["decimals"].as_u64().unwrap_or(18) as u8;
        return Ok(TokenInfo { address, decimals });
    }

    bail!("token '{}' not found on chain {}. Use contract address directly.", input, chain_id)
}

async fn fetch_decimals(client: &RelayClient, address: &str, chain_id: u64) -> Result<u8> {
    let body = json!({
        "chainIds": [chain_id],
        "address": address,
        "limit": 1,
    });
    let url = client.url("/currencies/v1");
    let results: Vec<Vec<serde_json::Value>> = client
        .http
        .post(&url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    let decimals = results.iter().flatten().next()
        .and_then(|t| t["decimals"].as_u64())
        .unwrap_or(18) as u8;
    Ok(decimals)
}

/// Convert human-readable amount (e.g. "0.001") to raw integer string using token decimals.
pub fn to_raw(amount: &str, decimals: u8) -> Result<String> {
    let (whole_str, frac_str) = match amount.split_once('.') {
        Some((w, f)) => (w, f),
        None => (amount, ""),
    };

    if frac_str.len() > decimals as usize {
        bail!(
            "amount '{}' has more decimal places ({}) than token supports ({})",
            amount, frac_str.len(), decimals
        );
    }

    // Pad fraction to exactly `decimals` digits
    let frac_padded = format!("{:0<width$}", frac_str, width = decimals as usize);

    let whole: u128 = if whole_str.is_empty() { 0 } else { whole_str.parse()? };
    let frac: u128 = if frac_padded.is_empty() { 0 } else { frac_padded.parse()? };
    let scale = 10u128.pow(decimals as u32);

    Ok((whole * scale + frac).to_string())
}
