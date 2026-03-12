use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryActionKind {
    RegisterParticipant,
    TransitionParticipant,
    AddAttestation,
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
