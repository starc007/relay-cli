use anyhow::Result;
use serde_json::json;
use crate::lib::client::RelayClient;

const NATIVE_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

pub async fn resolve(client: &RelayClient, input: &str, chain_id: u64) -> Result<String> {
    // Already an address
    if input.starts_with("0x") && input.len() == 42 {
        return Ok(input.to_string());
    }

    // Native token shorthands
    let upper = input.to_uppercase();
    if matches!(upper.as_str(), "ETH" | "MATIC" | "BNB" | "AVAX" | "FTM" | "ONE" | "CELO" | "METIS") {
        return Ok(NATIVE_ADDRESS.to_string());
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
        if let Some(addr) = token["address"].as_str() {
            return Ok(addr.to_string());
        }
    }

    anyhow::bail!("token '{}' not found on chain {}. Use contract address directly.", input, chain_id)
}
