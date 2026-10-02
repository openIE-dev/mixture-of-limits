//! HMAC-SHA256 signing stub (leapfrog-style honesty — not ed25519 PKI).

use hmac::{Hmac, Mac};
use mol_core::MolError;
use sha2::Sha256;

use crate::receipt::MolReceipt;

/// Env var for HMAC secret (optional).
pub const HMAC_SECRET_ENV: &str = "MOL_RECEIPT_SECRET";

/// Algorithm label written on receipts.
pub const SIGNATURE_ALG: &str = "hmac-sha256";

type HmacSha256 = Hmac<Sha256>;

/// Canonical bytes for signing (stable field subset — excludes signature itself).
pub fn canonical_bytes(r: &MolReceipt) -> Result<Vec<u8>, serde_json::Error> {
    // Sign a redacted clone without signature fields.
    let mut clone = r.clone();
    clone.signature_hex = None;
    clone.signature_alg = None;
    serde_json::to_vec(&clone)
}

/// Sign receipt in place with the given secret.
pub fn sign_receipt(r: &mut MolReceipt, secret: &[u8]) -> Result<(), MolError> {
    let bytes = canonical_bytes(r).map_err(|e| MolError::Msg(e.to_string()))?;
    let mut mac =
        HmacSha256::new_from_slice(secret).map_err(|e| MolError::Msg(format!("hmac key: {e}")))?;
    mac.update(&bytes);
    let sig = mac.finalize().into_bytes();
    r.signature_hex = Some(hex::encode(sig));
    r.signature_alg = Some(SIGNATURE_ALG.into());
    Ok(())
}

/// Verify signature with secret.
pub fn verify_signature(r: &MolReceipt, secret: &[u8]) -> Result<(), MolError> {
    let Some(sig_hex) = r.signature_hex.as_deref() else {
        return Err(MolError::ReceiptVerify("missing signature".into()));
    };
    if r.signature_alg.as_deref() != Some(SIGNATURE_ALG) {
        return Err(MolError::ReceiptVerify(format!(
            "unexpected alg {:?}",
            r.signature_alg
        )));
    }
    let bytes = canonical_bytes(r).map_err(|e| MolError::Msg(e.to_string()))?;
    let mut mac =
        HmacSha256::new_from_slice(secret).map_err(|e| MolError::Msg(format!("hmac key: {e}")))?;
    mac.update(&bytes);
    let expected = mac.finalize().into_bytes();
    let got = hex::decode(sig_hex).map_err(|e| MolError::ReceiptVerify(format!("hex: {e}")))?;
    if expected.as_slice() != got.as_slice() {
        return Err(MolError::ReceiptVerify("HMAC mismatch".into()));
    }
    Ok(())
}

/// Load secret from env or return None.
pub fn secret_from_env() -> Option<Vec<u8>> {
    std::env::var(HMAC_SECRET_ENV)
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| s.into_bytes())
}
