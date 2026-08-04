//! Shared ledger primitives used by every state-mutating registry operation.
//!
//! `WriteContext` captures the provenance of a write (local vs on-chain), and
//! the hashing helpers produce the deterministic transaction hashes and content
//! digests that anchor records. Centralising these avoids drift between the
//! participant registry and the dispute registry.

use sha2::{Digest, Sha256};

/// Provenance for a single state mutation.
///
/// In `local` mode the framework simulates a devnet (deterministic tx hashes,
/// monotonic block heights). In `chain` mode the fields are populated from the
/// broadcast receipt of a pre-signed transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteContext {
    pub source: String,
    pub tx_hash: String,
    pub block: u64,
    pub network: Option<String>,
    pub chain_id: Option<u64>,
    pub contract: Option<String>,
}

impl WriteContext {
    /// Context for an in-memory/local devnet write.
    pub fn local() -> Self {
        WriteContext {
            source: "local".to_string(),
            tx_hash: String::new(),
            block: 0,
            network: None,
            chain_id: None,
            contract: None,
        }
    }

    /// Whether this write did not originate from a broadcast chain transaction.
    pub fn is_local(&self) -> bool {
        self.tx_hash.is_empty()
    }
}

impl Default for WriteContext {
    fn default() -> Self {
        WriteContext::local()
    }
}

/// Lowercase hex-encoded SHA-256 of `input`.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Deterministic pseudo transaction hash used for local-mode writes.
pub fn tx_hash(seed: &str) -> String {
    format!("0x{}", &sha256_hex(seed)[..32])
}
