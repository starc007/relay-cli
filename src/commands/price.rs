use anyhow::Result;
use serde::Deserialize;
use crate::lib::{client::RelayClient, token};

#[derive(Debug, Deserialize)]
struct PriceResponse {
    price: Option<f64>,
}

pub async fn run(client: &RelayClient, input: &str, chain_id: u64) -> Result<()> {
    let address = token::resolve(client, input, chain_id).await?;
    let url = client.url(&format!("/currencies/token/price?address={}&chainId={}", address, chain_id));
    let resp: PriceResponse = client.http.get(&url).send().await?.json().await?;

    match resp.price {
        Some(p) => println!("{} (chain {}): ${:.6}", input, chain_id, p),
        None => println!("price not available"),
    }

    Ok(())
}
