//! Strongly-typed dispute domain vocabulary.
//!
//! Reason codes, dispute statuses, and resolution decisions are modelled as
//! closed enums rather than free strings. This makes illegal states
//! unrepresentable in the type system while preserving the exact snake_case
//! wire form used by the JSON schemas and persisted registry state.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Error returned when parsing a domain enum from an unknown string value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseEnumError {
    pub kind: &'static str,
    pub value: String,
    pub allowed: &'static str,
}

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown {} '{}'. Allowed: {}",
            self.kind, self.value, self.allowed
        )
    }
}

impl std::error::Error for ParseEnumError {}

/// Fixed dispute reason codes. Free-text statements are supplementary and
/// settlement logic MUST NOT depend on interpreting prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    TermsMismatch,
    MissingEvidence,
    LateFulfillment,
    DuplicateCharge,
    NonDelivery,
}

impl ReasonCode {
    pub const ALL: [ReasonCode; 5] = [
        ReasonCode::TermsMismatch,
        ReasonCode::MissingEvidence,
        ReasonCode::LateFulfillment,
        ReasonCode::DuplicateCharge,
        ReasonCode::NonDelivery,
    ];
    const ALLOWED: &'static str =
        "terms_mismatch, missing_evidence, late_fulfillment, duplicate_charge, non_delivery";

    pub fn as_str(self) -> &'static str {
        match self {
            ReasonCode::TermsMismatch => "terms_mismatch",
            ReasonCode::MissingEvidence => "missing_evidence",
            ReasonCode::LateFulfillment => "late_fulfillment",
            ReasonCode::DuplicateCharge => "duplicate_charge",
            ReasonCode::NonDelivery => "non_delivery",
        }
    }
}

impl fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ReasonCode {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ReasonCode::ALL
            .into_iter()
            .find(|c| c.as_str() == s)
            .ok_or(ParseEnumError {
                kind: "reason_code",
                value: s.to_string(),
                allowed: ReasonCode::ALLOWED,
            })
    }
}

/// Lifecycle status of a dispute record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisputeStatus {
    Open,
    Investigating,
    Resolved,
    Closed,
}

impl DisputeStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            DisputeStatus::Open => "open",
            DisputeStatus::Investigating => "investigating",
            DisputeStatus::Resolved => "resolved",
            DisputeStatus::Closed => "closed",
        }
    }

    /// Whether the dispute is still active (accepts evidence / can be resolved).
    pub fn is_active(self) -> bool {
        matches!(self, DisputeStatus::Open | DisputeStatus::Investigating)
    }

    /// Whether the dispute has reached a terminal state.
    pub fn is_terminal(self) -> bool {
        matches!(self, DisputeStatus::Resolved | DisputeStatus::Closed)
    }
}

impl fmt::Display for DisputeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Terminal resolution decisions a dispute may be resolved with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionDecision {
    Fulfilled,
    Failed,
    Refunded,
}

impl ResolutionDecision {
    pub const ALL: [ResolutionDecision; 3] = [
        ResolutionDecision::Fulfilled,
        ResolutionDecision::Failed,
        ResolutionDecision::Refunded,
    ];
    const ALLOWED: &'static str = "fulfilled, failed, refunded";

    pub fn as_str(self) -> &'static str {
        match self {
            ResolutionDecision::Fulfilled => "fulfilled",
            ResolutionDecision::Failed => "failed",
            ResolutionDecision::Refunded => "refunded",
        }
    }
}

impl fmt::Display for ResolutionDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ResolutionDecision {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ResolutionDecision::ALL
            .into_iter()
            .find(|d| d.as_str() == s)
            .ok_or(ParseEnumError {
                kind: "decision",
                value: s.to_string(),
                allowed: ResolutionDecision::ALLOWED,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_code_roundtrips_through_wire_form() {
        for code in ReasonCode::ALL {
            assert_eq!(ReasonCode::from_str(code.as_str()).unwrap(), code);
        }
        assert!(ReasonCode::from_str("nope").is_err());
    }

    #[test]
    fn decision_roundtrips_through_wire_form() {
        for decision in ResolutionDecision::ALL {
            assert_eq!(
                ResolutionDecision::from_str(decision.as_str()).unwrap(),
                decision
            );
        }
        assert!(ResolutionDecision::from_str("disputed").is_err());
    }

    #[test]
    fn status_activity_predicates() {
        assert!(DisputeStatus::Open.is_active());
        assert!(DisputeStatus::Investigating.is_active());
        assert!(DisputeStatus::Resolved.is_terminal());
        assert!(DisputeStatus::Closed.is_terminal());
    }
}
