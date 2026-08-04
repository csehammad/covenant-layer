use crate::constants::allowed_targets;
use crate::ledger::{sha256_hex, tx_hash};
use crate::manifest::{
    compute_manifest_hash, load_manifest, manifest_field_string, validate_manifest,
};
use crate::models::{Attestation, ParticipantRecord};
use crate::store::{load_state, next_block, save_state, utc_now};
use serde_json::json;

pub use crate::ledger::WriteContext;

pub fn register_participant(
    manifest_path: &str,
    owner: &str,
    stake: f64,
) -> Result<ParticipantRecord, String> {
    register_participant_with_context(manifest_path, owner, stake, &WriteContext::local())
}

pub fn register_participant_with_context(
    manifest_path: &str,
    owner: &str,
    stake: f64,
    context: &WriteContext,
) -> Result<ParticipantRecord, String> {
    let manifest = load_manifest(manifest_path)?;
    let validation_errors = validate_manifest(&manifest);
    if !validation_errors.is_empty() {
        return Err(format!(
            "Manifest validation failed: {}",
            validation_errors.join("; ")
        ));
    }

    let participant_id = manifest_field_string(&manifest, "participant_id")?;
    let participant_type = manifest_field_string(&manifest, "participant_type")?;
    let manifest_id = manifest_field_string(&manifest, "manifest_id")?;

    let mut state = load_state()?;
    if state.participants.contains_key(&participant_id) {
        return Err(format!("participant already registered: {participant_id}"));
    }

    let block = if context.block > 0 {
        context.block
    } else {
        next_block(&mut state)
    };
    let hash = compute_manifest_hash(manifest_path)?;
    let tx = if context.tx_hash.is_empty() {
        tx_hash(&format!("{participant_id}:{block}:register"))
    } else {
        context.tx_hash.clone()
    };
    let now = utc_now();

    let record = ParticipantRecord {
        participant_id: participant_id.clone(),
        participant_type,
        owner_address: owner.to_string(),
        manifest_path: manifest_path.to_string(),
        manifest_id,
        manifest_hash: hash,
        stake,
        state: "requested".to_string(),
        attestations: vec![],
        registered_at: now.clone(),
        updated_at: now,
        last_tx_hash: tx.clone(),
        last_block: block,
        last_source: context.source.clone(),
        last_network: context.network.clone(),
        last_chain_id: context.chain_id,
        last_contract: context.contract.clone(),
    };

    state
        .participants
        .insert(participant_id.clone(), record.clone());
    state.events.push(json!({
        "type": "register_participant",
        "participant_id": participant_id,
        "state": "requested",
        "source": context.source.clone(),
        "network": context.network.clone(),
        "chain_id": context.chain_id,
        "contract": context.contract.clone(),
        "tx_hash": tx,
        "block": block,
        "timestamp": utc_now()
    }));
    save_state(&state)?;
    Ok(record)
}

pub fn transition_participant(
    participant_id: &str,
    to_state: &str,
    reason: &str,
) -> Result<ParticipantRecord, String> {
    transition_participant_with_context(participant_id, to_state, reason, &WriteContext::local())
}

pub fn transition_participant_with_context(
    participant_id: &str,
    to_state: &str,
    reason: &str,
    context: &WriteContext,
) -> Result<ParticipantRecord, String> {
    let mut state = load_state()?;
    let from_state = state
        .participants
        .get(participant_id)
        .ok_or_else(|| format!("unknown participant: {participant_id}"))?
        .state
        .clone();
    if !allowed_targets(&from_state).contains(&to_state) {
        return Err(format!(
            "Invalid transition '{}' -> '{}'. Allowed targets: {:?}",
            from_state,
            to_state,
            allowed_targets(&from_state)
        ));
    }

    let block = if context.block > 0 {
        context.block
    } else {
        next_block(&mut state)
    };
    let tx = if context.tx_hash.is_empty() {
        tx_hash(&format!("{participant_id}:{from_state}:{to_state}:{block}"))
    } else {
        context.tx_hash.clone()
    };
    let participant = state
        .participants
        .get_mut(participant_id)
        .ok_or_else(|| format!("unknown participant: {participant_id}"))?;
    participant.state = to_state.to_string();
    participant.updated_at = utc_now();
    participant.last_tx_hash = tx.clone();
    participant.last_block = block;
    participant.last_source = context.source.clone();
    participant.last_network = context.network.clone();
    participant.last_chain_id = context.chain_id;
    participant.last_contract = context.contract.clone();
    let updated = participant.clone();

    state.events.push(json!({
        "type": "transition_participant",
        "participant_id": participant_id,
        "from": from_state,
        "to": to_state,
        "reason": reason,
        "source": context.source.clone(),
        "network": context.network.clone(),
        "chain_id": context.chain_id,
        "contract": context.contract.clone(),
        "tx_hash": tx,
        "block": block,
        "timestamp": utc_now()
    }));
    save_state(&state)?;
    Ok(updated)
}

pub fn add_attestation(
    participant_id: &str,
    kind: &str,
    details: serde_json::Value,
) -> Result<Attestation, String> {
    add_attestation_with_context(participant_id, kind, details, &WriteContext::local())
}

pub fn add_attestation_with_context(
    participant_id: &str,
    kind: &str,
    details: serde_json::Value,
    context: &WriteContext,
) -> Result<Attestation, String> {
    let mut state = load_state()?;
    let block = if context.block > 0 {
        context.block
    } else {
        next_block(&mut state)
    };
    let attestation_id = format!(
        "att_{}",
        &sha256_hex(&format!("{participant_id}:{block}"))[..12]
    );
    let att = Attestation {
        attestation_id: attestation_id.clone(),
        r#type: kind.to_string(),
        details,
        timestamp: utc_now(),
    };

    let participant = state
        .participants
        .get_mut(participant_id)
        .ok_or_else(|| format!("unknown participant: {participant_id}"))?;
    participant.attestations.push(att.clone());
    participant.updated_at = utc_now();
    participant.last_block = block;
    participant.last_tx_hash = if context.tx_hash.is_empty() {
        tx_hash(&format!("{participant_id}:{attestation_id}:{block}"))
    } else {
        context.tx_hash.clone()
    };
    participant.last_source = context.source.clone();
    participant.last_network = context.network.clone();
    participant.last_chain_id = context.chain_id;
    participant.last_contract = context.contract.clone();

    state.events.push(json!({
        "type": "attestation_added",
        "participant_id": participant_id,
        "attestation_id": attestation_id,
        "source": context.source.clone(),
        "network": context.network.clone(),
        "chain_id": context.chain_id,
        "contract": context.contract.clone(),
        "block": block,
        "timestamp": utc_now()
    }));
    save_state(&state)?;
    Ok(att)
}

pub fn get_participant(participant_id: &str) -> Result<ParticipantRecord, String> {
    let state = load_state()?;
    state
        .participants
        .get(participant_id)
        .cloned()
        .ok_or_else(|| format!("unknown participant: {participant_id}"))
}

pub fn list_participants() -> Result<Vec<ParticipantRecord>, String> {
    let state = load_state()?;
    Ok(state.participants.values().cloned().collect())
}
