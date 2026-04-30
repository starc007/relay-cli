use anyhow::Result;
use serde::Deserialize;
use crate::lib::client::RelayClient;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrencyInfo {
    symbol: Option<String>,
    chain_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrencyAmount {
    currency: Option<CurrencyInfo>,
    amount_formatted: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestData {
    currency_in: Option<CurrencyAmount>,
    currency_out: Option<CurrencyAmount>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    id: Option<String>,
    status: Option<String>,
    created_at: Option<u64>,
    data: Option<RequestData>,
}

#[derive(Debug, Deserialize)]
struct RequestsResponse {
    requests: Option<Vec<Request>>,
}

pub async fn run(
    client: &RelayClient,
    user: &str,
    limit: u32,
    status: Option<&str>,
) -> Result<()> {
    let mut url = client.url(&format!("/requests/v2?user={}&limit={}", user, limit));
    if let Some(s) = status {
        url.push_str(&format!("&status={}", s));
    }

    let resp: RequestsResponse = client.http.get(&url).send().await?.json().await?;
    let requests = resp.requests.unwrap_or_default();

    if requests.is_empty() {
        println!("no requests found");
        return Ok(());
    }

    println!("{:<20} {:<12} {:<30} {}", "TIME", "STATUS", "ROUTE", "ID");
    println!("{}", "-".repeat(100));

    for req in &requests {
        let time = req.created_at
            .map(|t| format_ts(t))
            .unwrap_or_else(|| "-".to_string());

        let status = req.status.as_deref().unwrap_or("-");

        let route = req.data.as_ref().map(|d| {
            let cin = d.currency_in.as_ref().map(|c| {
                format!(
                    "{} {}({})",
                    c.amount_formatted.as_deref().unwrap_or("?"),
                    c.currency.as_ref().and_then(|x| x.symbol.as_deref()).unwrap_or("?"),
                    c.currency.as_ref().and_then(|x| x.chain_id).map(|id| id.to_string()).as_deref().unwrap_or("?"),
                )
            }).unwrap_or_default();
            let cout = d.currency_out.as_ref().map(|c| {
                format!(
                    "{} {}({})",
                    c.amount_formatted.as_deref().unwrap_or("?"),
                    c.currency.as_ref().and_then(|x| x.symbol.as_deref()).unwrap_or("?"),
                    c.currency.as_ref().and_then(|x| x.chain_id).map(|id| id.to_string()).as_deref().unwrap_or("?"),
                )
            }).unwrap_or_default();
            format!("{} → {}", cin, cout)
        }).unwrap_or_else(|| "-".to_string());

        let id = req.id.as_deref().unwrap_or("-");
        let short_id = if id.len() > 18 { &id[..18] } else { id };

        println!("{:<20} {:<12} {:<30} {}", time, status, route, short_id);
    }

    Ok(())
}

fn format_ts(ts: u64) -> String {
    use chrono::{DateTime, Utc};
    DateTime::<Utc>::from_timestamp(ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".to_string())
}
