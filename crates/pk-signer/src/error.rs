use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignerError {
    #[error("Invalid private key: {0}")]
    InvalidKey(String),

    #[error("EIP-712 encoding failed: {0}")]
    Encoding(String),

    #[error("Signing failed: {0}")]
    Signing(String),

    #[error("POLYMARKET_PRIVATE_KEY not set — trading disabled")]
    NoKey,
}
