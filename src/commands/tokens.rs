use anyhow::Result;
use serde::Deserialize;
use serde_json::json;
use crate::lib::client::RelayClient;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenMetadata {
    verified: Option<bool>,
    is_native: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Token {
    chain_id: Option<u64>,
    address: Option<String>,
    symbol: Option<String>,
    name: Option<String>,
    decimals: Option<u8>,
    metadata: Option<TokenMetadata>,
}

pub async fn run(
    client: &RelayClient,
    chain_id: u64,
    filter: Option<&str>,
    verified_only: bool,
) -> Result<()> {
    let body = json!({
        "chainIds": [chain_id],
        "verified": if verified_only { Some(true) } else { None },
        "limit": 50,
    });

    let url = client.url("/currencies/v1");
    let tokens: Vec<Vec<Token>> = client.http.post(&url).json(&body).send().await?.json().await?;
    let tokens: Vec<&Token> = tokens.iter().flatten().collect();

    let tokens: Vec<&&Token> = tokens.iter().filter(|t| {
        filter.map_or(true, |f| {
            let f = f.to_lowercase();
            t.symbol.as_deref().map(|s| s.to_lowercase().contains(&f)).unwrap_or(false)
                || t.name.as_deref().map(|n| n.to_lowercase().contains(&f)).unwrap_or(false)
        })
    }).collect();

    println!("{:<12} {:<46} {:<8} {}", "SYMBOL", "ADDRESS", "DECIMALS", "NAME");
    println!("{}", "-".repeat(90));
    for t in tokens {
        println!(
            "{:<12} {:<46} {:<8} {}",
            t.symbol.as_deref().unwrap_or("-"),
            t.address.as_deref().unwrap_or("-"),
            t.decimals.map(|d| d.to_string()).as_deref().unwrap_or("-"),
            t.name.as_deref().unwrap_or("-"),
        );
    }

    Ok(())
}
