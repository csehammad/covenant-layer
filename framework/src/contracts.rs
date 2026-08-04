use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryActionKind {
    RegisterParticipant,
    TransitionParticipant,
    AddAttestation,
    OpenDispute,
    ResolveDispute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryAction {
    pub kind: RegistryActionKind,
    pub participant_id: String,
    pub payload: serde_json::Value,
}

pub fn register_action(
    participant_id: &str,
    manifest_hash: &str,
    owner: &str,
    stake: f64,
) -> RegistryAction {
    RegistryAction {
        kind: RegistryActionKind::RegisterParticipant,
        participant_id: participant_id.to_string(),
        payload: json!({
            "manifest_hash": manifest_hash,
            "owner": owner,
            "stake": stake
        }),
    }
}

pub fn transition_action(
    participant_id: &str,
    from: &str,
    to: &str,
    reason: &str,
) -> RegistryAction {
    RegistryAction {
        kind: RegistryActionKind::TransitionParticipant,
        participant_id: participant_id.to_string(),
        payload: json!({
            "from": from,
            "to": to,
            "reason": reason
        }),
    }
}

pub fn attestation_action(participant_id: &str, kind: &str) -> RegistryAction {
    RegistryAction {
        kind: RegistryActionKind::AddAttestation,
        participant_id: participant_id.to_string(),
        payload: json!({
            "attestation_type": kind
        }),
    }
}

pub fn open_dispute_action(
    dispute_id: &str,
    commitment_ref: &str,
    evidence_root: &str,
    response_deadline: &str,
) -> RegistryAction {
    RegistryAction {
        kind: RegistryActionKind::OpenDispute,
        participant_id: dispute_id.to_string(),
        payload: json!({
            "commitment_ref": commitment_ref,
            "evidence_root": evidence_root,
            "response_deadline": response_deadline
        }),
    }
}

pub fn resolve_dispute_action(
    dispute_id: &str,
    decision: &str,
    resolution_evidence_root: &str,
) -> RegistryAction {
    RegistryAction {
        kind: RegistryActionKind::ResolveDispute,
        participant_id: dispute_id.to_string(),
        payload: json!({
            "decision": decision,
            "resolution_evidence_root": resolution_evidence_root
        }),
    }
}
