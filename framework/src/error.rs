//! Typed error taxonomy for the dispute subsystem.
//!
//! Callers can match on specific failure modes instead of string-scraping.
//! The `From<String>`/`Into<String>` bridges keep interop with the rest of the
//! framework (which still surfaces `Result<_, String>` at the CLI boundary)
//! seamless while `?` propagation stays ergonomic.

use thiserror::Error;

/// Errors that can arise while recording commitments or driving the dispute
/// lifecycle.
#[derive(Debug, Error)]
pub enum DisputeError {
    /// Underlying state store (I/O or (de)serialization) failure.
    #[error("{0}")]
    Store(String),

    #[error("commitment_ref is required")]
    MissingCommitmentRef,

    #[error("commitment requires accepted_by and provider_id parties")]
    MissingParties,

    #[error("commitment already recorded: {0}")]
    CommitmentExists(String),

    #[error("unknown commitment: {0}")]
    UnknownCommitment(String),

    #[error("unknown dispute: {0}")]
    UnknownDispute(String),

    #[error("unknown reason_code '{value}'. Allowed: {allowed}")]
    UnknownReasonCode { value: String, allowed: String },

    #[error("invalid decision '{value}'. Allowed: {allowed}")]
    InvalidDecision { value: String, allowed: String },

    #[error("at least one evidence reference is required")]
    EvidenceRequired,

    #[error("invalid {field} timestamp '{value}': {reason}")]
    InvalidTimestamp {
        field: String,
        value: String,
        reason: String,
    },

    #[error("'{party}' is not a party to commitment '{commitment}'")]
    NotACommitmentParty { party: String, commitment: String },

    #[error("commitment '{commitment}' is not disputable in state '{state}'")]
    NotDisputable { commitment: String, state: String },

    #[error("dispute already exists (replay): {0}")]
    DuplicateDispute(String),

    #[error("an open dispute already exists for commitment '{0}'")]
    OpenDisputeExists(String),

    #[error("dispute '{dispute}' is {status} and cannot accept evidence")]
    DisputeNotOpenForEvidence { dispute: String, status: String },

    #[error("evidence is immutable after response_deadline for dispute '{0}'")]
    EvidenceImmutable(String),

    #[error("dispute '{dispute}' is already {status} and cannot be re-resolved")]
    AlreadyResolved { dispute: String, status: String },

    #[error(
        "'{party}' is not authorized to resolve dispute '{dispute}' (requires assigned verifier '{verifier}' or settlement authority '{authority}')"
    )]
    Unauthorized {
        party: String,
        dispute: String,
        verifier: String,
        authority: String,
    },
}

impl From<String> for DisputeError {
    fn from(message: String) -> Self {
        DisputeError::Store(message)
    }
}

impl From<DisputeError> for String {
    fn from(error: DisputeError) -> Self {
        error.to_string()
    }
}

/// Convenience alias for fallible dispute operations.
pub type DisputeResult<T> = Result<T, DisputeError>;
