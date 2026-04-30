use anyhow::Result;
use reqwest::{Client, header};

const MAINNET_API: &str = "https://api.relay.link";
const TESTNET_API: &str = "https://api.testnets.relay.link";

pub struct RelayClient {
    pub http: Client,
    pub base_url: String,
}

impl RelayClient {
    pub fn new(api_key: Option<&str>, testnet: bool) -> Result<Self> {
        let base_url = if testnet {
            TESTNET_API.to_string()
        } else {
            MAINNET_API.to_string()
        };

        let mut headers = header::HeaderMap::new();
        if let Some(key) = api_key {
            headers.insert(
                "x-api-key",
                header::HeaderValue::from_str(key)?,
            );
        }

        let http = Client::builder()
            .default_headers(headers)
            .user_agent(concat!("relay-cli/", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self { http, base_url })
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}
