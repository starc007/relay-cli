use anyhow::Result;
use crate::lib::client::RelayClient;
use crate::lib::types::ChainsResponse;

pub async fn run(client: &RelayClient, filter: Option<&str>) -> Result<()> {
    let url = client.url("/chains");
    let resp: ChainsResponse = client.http.get(&url).send().await?.json().await?;

    let chains = resp.chains.iter().filter(|c| {
        filter.map_or(true, |f| {
            c.name.to_lowercase().contains(&f.to_lowercase())
                || c.id.to_string() == f
        })
    });

    println!("{:<10} {}", "CHAIN ID", "NAME");
    println!("{}", "-".repeat(40));
    for chain in chains {
        println!("{:<10} {}", chain.id, chain.display_name.as_deref().unwrap_or(&chain.name));
    }

    Ok(())
}
