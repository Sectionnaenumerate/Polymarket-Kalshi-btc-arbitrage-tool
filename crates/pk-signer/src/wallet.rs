use ethers::signers::{LocalWallet, Signer};
use std::str::FromStr;

use crate::error::SignerError;

/// EVM wallet configured for Polymarket CLOB signing (Polygon / chain 137).
pub struct PolyWallet {
    pub inner: LocalWallet,
    pub address: String,
    /// Optional Gnosis Safe / proxy wallet address
    pub proxy_address: Option<String>,
    pub chain_id: u64,
}

impl PolyWallet {
    pub fn from_key(private_key: &str, chain_id: u64, proxy: Option<String>) -> Result<Self, SignerError> {
        let key = private_key.trim_start_matches("0x");
        let wallet = LocalWallet::from_str(key)
            .map_err(|e| SignerError::InvalidKey(e.to_string()))?
            .with_chain_id(chain_id);
        let address = format!("{:#x}", wallet.address());
        Ok(Self { inner: wallet, address, proxy_address: proxy, chain_id })
    }

    /// Load from POLYMARKET_PRIVATE_KEY env var.
    pub fn from_env() -> Result<Self, SignerError> {
        let key = std::env::var("POLYMARKET_PRIVATE_KEY").map_err(|_| SignerError::NoKey)?;
        let chain_id: u64 = std::env::var("POLYMARKET_CHAIN_ID")
            .unwrap_or_else(|_| "137".into())
            .parse()
            .unwrap_or(137);
        let proxy = std::env::var("POLYMARKET_PROXY_WALLET_ADDRESS").ok();
        Self::from_key(&key, chain_id, proxy)
    }

    pub fn effective_address(&self) -> &str {
        self.proxy_address.as_deref().unwrap_or(&self.address)
    }
}

impl std::fmt::Debug for PolyWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PolyWallet(eoa={}, proxy={:?})", self.address, self.proxy_address)
    }
}
