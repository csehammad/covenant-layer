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
pub struct RegistryState {
    pub chain: ChainState,
    pub participants: HashMap<String, ParticipantRecord>,
    pub events: Vec<Value>,
}
