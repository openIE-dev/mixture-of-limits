//! MoL receipts — joule brackets, cascade steps, HMAC signing stub.
//!
//! Honesty contract (matches openie-leapfrog / WCA):
//! - `estimated_j` is always labeled (analytical / cascade table).
//! - `measured_j` is `Option` — **never invented RAPL/NVML**.
//! - `board_synth_claimed = false`.
//! - Signing is HMAC-SHA256 stub (leapfrog-style honesty), not a PKI claim.

#![deny(missing_docs)]

mod receipt;
mod sign;
mod verify;
mod transcript;

pub use receipt::{
    CascadeStepOutcome, CascadeStepRecord, MolReceipt, ReceiptBuilder, SynthesisReceipt,
};
pub use sign::{
    canonical_bytes, secret_from_env, sign_receipt, verify_signature, HMAC_SECRET_ENV,
    SIGNATURE_ALG,
};
pub use verify::verify_receipt_integrity;
pub use transcript::{
    replay_transcript, CloseFingerprint, CloseTranscript, CloseTranscriptEntry, ReplayCloseOutcome,
    ReplayEntryReport, ReplayReport,
};

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
