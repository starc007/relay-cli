use anyhow::{bail, Context, Result};
use alloy::signers::local::PrivateKeySigner;

pub fn load_signer(key_override: Option<&str>) -> Result<PrivateKeySigner> {
    let raw = if let Some(k) = key_override {
        k.to_string()
    } else {
        std::env::var("RELAY_PRIVATE_KEY")
            .context("no wallet: set RELAY_PRIVATE_KEY env or use --private-key")?
    };

    let raw = raw.trim_start_matches("0x");
    if raw.len() != 64 {
        bail!("private key must be 32 bytes (64 hex chars)");
    }

    raw.parse::<PrivateKeySigner>().context("invalid private key")
}
