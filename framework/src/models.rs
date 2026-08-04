use crate::domain::{DisputeStatus, ReasonCode, ResolutionDecision};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attestation {
    pub attestation_id: String,
    pub r#type: String,
    pub details: Value,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParticipantRecord {
    pub participant_id: String,
    pub participant_type: String,
    pub owner_address: String,
    pub manifest_path: String,
    pub manifest_id: String,
    pub manifest_hash: String,
    pub stake: f64,
    pub state: String,
    pub attestations: Vec<Attestation>,
    pub registered_at: String,
    pub updated_at: String,
    pub last_tx_hash: String,
    pub last_block: u64,
    pub last_source: String,
    pub last_network: Option<String>,
    pub last_chain_id: Option<u64>,
    pub last_contract: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainState {
    pub name: String,
    pub block_height: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvidenceRef {
    pub r#type: String,
    pub digest: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommitmentRecord {
    pub commitment_ref: String,
    pub objective_ref: String,
    pub offer_id: String,
    pub accepted_by: String,
    pub provider_id: String,
    pub assigned_verifier: String,
    #[serde(default)]
    pub settlement_authority: String,
    pub settlement_state: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_tx_hash: String,
    pub last_block: u64,
    pub last_source: String,
    pub last_network: Option<String>,
    pub last_chain_id: Option<u64>,
    pub last_contract: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisputeResolution {
    pub resolution_id: String,
    pub dispute_id: String,
    pub commitment_ref: String,
    pub decision: ResolutionDecision,
    pub resolved_by: String,
    pub resolution_evidence_refs: Vec<EvidenceRef>,
    pub resolved_at: String,
    pub settlement_state: ResolutionDecision,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisputeRecord {
    pub dispute_id: String,
    pub commitment_ref: String,
    pub raised_by: String,
    pub reason_code: ReasonCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statement: Option<String>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub opened_at: String,
    pub response_deadline: String,
    pub status: DisputeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<DisputeResolution>,
    pub updated_at: String,
    pub last_tx_hash: String,
    pub last_block: u64,
    pub last_source: String,
    pub last_network: Option<String>,
    pub last_chain_id: Option<u64>,
    pub last_contract: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryState {
    pub chain: ChainState,
    pub participants: HashMap<String, ParticipantRecord>,
    pub events: Vec<Value>,
    #[serde(default)]
    pub commitments: HashMap<String, CommitmentRecord>,
    #[serde(default)]
    pub disputes: HashMap<String, DisputeRecord>,
}
