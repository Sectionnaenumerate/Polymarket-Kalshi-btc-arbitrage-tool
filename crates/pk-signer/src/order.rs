use ethers::{signers::Signer, types::H256, utils::keccak256};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{error::SignerError, wallet::PolyWallet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ClobOrderSide {
    Buy,
    Sell,
}

/// A Polymarket CLOB order before signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClobOrder {
    pub id: Uuid,
    pub token_id: String,
    pub side: ClobOrderSide,
    /// Price as 0–1 fraction (not cents)
    pub price: Decimal,
    /// Size in shares
    pub size: Decimal,
    /// "FOK" (fill-or-kill) or "GTC"
    pub time_in_force: String,
    pub nonce: u64,
}

impl ClobOrder {
    pub fn market_buy(token_id: impl Into<String>, price_frac: Decimal, size: Decimal) -> Self {
        Self {
            id: Uuid::new_v4(),
            token_id: token_id.into(),
            side: ClobOrderSide::Buy,
            price: price_frac,
            size,
            time_in_force: "FOK".to_string(),
            nonce: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Sign a CLOB order and return the JSON payload ready for POST /order.
pub async fn sign_clob_order(wallet: &PolyWallet, order: &ClobOrder) -> Result<Value, SignerError> {
    // Polymarket CLOB uses a simplified EIP-712 structure:
    // struct Order { address maker; address taker; ... }
    // We hash the canonical order fields and sign the digest.

    let maker = wallet.effective_address();
    let price_str = format!("{:.6}", order.price);
    let size_str = format!("{:.6}", order.size);

    // Build struct hash
    let type_hash = keccak256(
        b"Order(address maker,bytes32 tokenId,uint256 price,uint256 size,uint256 nonce,uint8 side)",
    );
    let mut buf = [0u8; 6 * 32];
    buf[..32].copy_from_slice(&type_hash);
    // remaining fields encoded as abi bytes — simplified for illustration
    let field_data = format!("{maker}{}{}{}{}", order.token_id, price_str, size_str, order.nonce);
    let field_hash = keccak256(field_data.as_bytes());
    buf[32..64].copy_from_slice(&field_hash);

    let struct_hash = keccak256(&buf[..64]);

    // Domain separator for Polygon (chain 137)
    let domain_hash = keccak256(
        format!("Polymarket CLOB v1 chain {}", wallet.chain_id).as_bytes(),
    );

    let mut digest_input = [0u8; 66];
    digest_input[0] = 0x19;
    digest_input[1] = 0x01;
    digest_input[2..34].copy_from_slice(&domain_hash);
    digest_input[34..66].copy_from_slice(&struct_hash);
    let digest = H256::from(keccak256(digest_input));

    let signature = wallet
        .inner
        .sign_hash(digest)
        .map_err(|e| SignerError::Signing(e.to_string()))?;

    Ok(json!({
        "orderID": order.id.to_string(),
        "marketID": order.token_id,
        "side": order.side,
        "price": price_str,
        "size": size_str,
        "timeInForce": order.time_in_force,
        "nonce": order.nonce,
        "maker": maker,
        "signature": {
            "r": format!("0x{:064x}", signature.r),
            "s": format!("0x{:064x}", signature.s),
            "v": signature.v
        }
    }))
}
