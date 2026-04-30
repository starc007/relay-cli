use anyhow::{bail, Context, Result};
use alloy::signers::local::PrivateKeySigner;

pub fn load_signer(key_override: Option<&str>) -> Result<PrivateKeySigner> {
    let raw = if let Some(k) = key_override {
        k.to_string()
    } else {
        std::env::var("RELAY_PRIVATE_KEY").unwrap_or_default()
    };

    if raw.is_empty() {
        bail!(
            "no private key found\n  options:\n    relay config set --private-key 0x...\n    export RELAY_PRIVATE_KEY=0x...\n    relay bridge --private-key 0x..."
        );
    }

    let raw = raw.trim_start_matches("0x");
    if raw.len() != 64 {
        bail!("private key must be 32 bytes (64 hex chars), got {} chars", raw.len());
    }

    raw.parse::<PrivateKeySigner>().context("invalid private key — check it's a valid hex string")
}
