use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::time::Duration;
use crate::lib::client::RelayClient;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponse {
    status: Option<String>,
    details: Option<String>,
    in_tx_hashes: Option<Vec<String>>,
    tx_hashes: Option<Vec<String>>,
    origin_chain_id: Option<u64>,
    destination_chain_id: Option<u64>,
}

const TERMINAL: &[&str] = &["success", "failure", "refund", "fallback"];
const POLL_MS: u64 = 2000;

pub async fn run(client: &RelayClient, request_id: &str, watch: bool) -> Result<()> {
    let url = client.url(&format!("/intents/status/v2?requestId={}", request_id));

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(80));

    loop {
        let resp: StatusResponse = client.http.get(&url).send().await?.json().await?;
        let status = resp.status.as_deref().unwrap_or("unknown");

        spinner.set_message(format!("status: {}", status));

        let is_terminal = TERMINAL.contains(&status);

        if is_terminal || !watch {
            spinner.finish_and_clear();
            print_status(&resp, status);
            break;
        }

        tokio::time::sleep(Duration::from_millis(POLL_MS)).await;
    }

    Ok(())
}

fn print_status(resp: &StatusResponse, status: &str) {
    println!("status:  {}", status);
    if let Some(details) = &resp.details {
        if !details.is_empty() {
            println!("details: {}", details);
        }
    }
    if let (Some(from), Some(to)) = (resp.origin_chain_id, resp.destination_chain_id) {
        println!("route:   {} → {}", from, to);
    }
    if let Some(hashes) = &resp.in_tx_hashes {
        for h in hashes {
            println!("in tx:   {}", h);
        }
    }
    if let Some(hashes) = &resp.tx_hashes {
        for h in hashes {
            println!("out tx:  {}", h);
        }
    }
}
