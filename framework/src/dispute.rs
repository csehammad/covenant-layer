//! Dispute lifecycle: commitment recording, opening, evidence, and resolution.
//!
//! This module is the source of truth for dispute enforcement. It validates
//! authorization, disputability, replay protection, evidence immutability, and
//! resolution authority against a minimal commitment store persisted in the
//! registry state. All fallible operations return [`DisputeError`], and every
//! state mutation is stamped with a [`WriteContext`] for local/chain provenance.

use crate::constants::is_disputable_state;
use crate::domain::{DisputeStatus, ReasonCode, ResolutionDecision};
use crate::error::{DisputeError, DisputeResult};
use crate::ledger::{sha256_hex, tx_hash};
use crate::models::{CommitmentRecord, DisputeRecord, DisputeResolution, EvidenceRef};
use crate::store::{load_state, next_block, save_state, utc_now};
use chrono::{DateTime, Utc};
use serde_json::json;

pub use crate::ledger::WriteContext;

/// Parameters for seeding an accepted commitment.
#[derive(Debug, Clone, Copy)]
pub struct RecordCommitment<'a> {
    pub commitment_ref: &'a str,
    pub objective_ref: &'a str,
    pub offer_id: &'a str,
    pub accepted_by: &'a str,
    pub provider_id: &'a str,
    pub assigned_verifier: &'a str,
    pub settlement_authority: &'a str,
    pub settlement_state: &'a str,
}

/// Parameters for opening a dispute against a recorded commitment.
#[derive(Debug, Clone)]
pub struct OpenDispute<'a> {
    pub commitment_ref: &'a str,
    pub raised_by: &'a str,
    pub reason_code: &'a str,
    pub statement: Option<String>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub response_deadline: &'a str,
}

fn parse_rfc3339(value: &str, field: &str) -> DisputeResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| DisputeError::InvalidTimestamp {
            field: field.to_string(),
            value: value.to_string(),
            reason: e.to_string(),
        })
}

/// Register (seed) a commitment created by a valid acceptance. This is the
/// source of truth used to enforce dispute authorization and disputability.
pub fn record_commitment(req: &RecordCommitment) -> DisputeResult<CommitmentRecord> {
    record_commitment_with_context(req, &WriteContext::local())
}

pub fn record_commitment_with_context(
    req: &RecordCommitment,
    context: &WriteContext,
) -> DisputeResult<CommitmentRecord> {
    if req.commitment_ref.trim().is_empty() {
        return Err(DisputeError::MissingCommitmentRef);
    }
    if req.accepted_by.trim().is_empty() || req.provider_id.trim().is_empty() {
        return Err(DisputeError::MissingParties);
    }

    let mut state = load_state()?;
    if state.commitments.contains_key(req.commitment_ref) {
        return Err(DisputeError::CommitmentExists(req.commitment_ref.to_string()));
    }

    let block = resolve_block(context, &mut state);
    let tx = resolve_tx(
        context,
        &format!("{}:{}:commitment", req.commitment_ref, block),
    );
    let now = utc_now();

    let record = CommitmentRecord {
        commitment_ref: req.commitment_ref.to_string(),
        objective_ref: req.objective_ref.to_string(),
        offer_id: req.offer_id.to_string(),
        accepted_by: req.accepted_by.to_string(),
        provider_id: req.provider_id.to_string(),
        assigned_verifier: req.assigned_verifier.to_string(),
        settlement_authority: req.settlement_authority.to_string(),
        settlement_state: req.settlement_state.to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
        last_tx_hash: tx.clone(),
        last_block: block,
        last_source: context.source.clone(),
        last_network: context.network.clone(),
        last_chain_id: context.chain_id,
        last_contract: context.contract.clone(),
    };

    state
        .commitments
        .insert(req.commitment_ref.to_string(), record.clone());
    state.events.push(json!({
        "type": "commitment_recorded",
        "commitment_ref": req.commitment_ref,
        "accepted_by": req.accepted_by,
        "provider_id": req.provider_id,
        "settlement_state": req.settlement_state,
        "source": context.source,
        "network": context.network,
        "chain_id": context.chain_id,
        "contract": context.contract,
        "tx_hash": tx,
        "block": block,
        "timestamp": now
    }));
    save_state(&state)?;
    Ok(record)
}

pub fn get_commitment(commitment_ref: &str) -> DisputeResult<CommitmentRecord> {
    let state = load_state()?;
    state
        .commitments
        .get(commitment_ref)
        .cloned()
        .ok_or_else(|| DisputeError::UnknownCommitment(commitment_ref.to_string()))
}

/// Deterministic dispute id derived from the commitment and reason so the same
/// logical dispute request maps to the same identifier (replay detection).
pub fn derive_dispute_id(commitment_ref: &str, reason_code: &str) -> String {
    format!(
        "dsp_{}",
        &sha256_hex(&format!("{commitment_ref}:{reason_code}"))[..24]
    )
}

pub fn open_dispute(req: OpenDispute) -> DisputeResult<DisputeRecord> {
    open_dispute_with_context(req, &WriteContext::local())
}

pub fn open_dispute_with_context(
    req: OpenDispute,
    context: &WriteContext,
) -> DisputeResult<DisputeRecord> {
    let reason: ReasonCode =
        req.reason_code
            .parse()
            .map_err(|_| DisputeError::UnknownReasonCode {
                value: req.reason_code.to_string(),
                allowed: ReasonCode::ALL
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            })?;
    if req.evidence_refs.is_empty() {
        return Err(DisputeError::EvidenceRequired);
    }
    // Validate the deadline is a well-formed RFC3339 timestamp up front.
    parse_rfc3339(req.response_deadline, "response_deadline")?;

    let mut state = load_state()?;

    let commitment = state
        .commitments
        .get(req.commitment_ref)
        .cloned()
        .ok_or_else(|| DisputeError::UnknownCommitment(req.commitment_ref.to_string()))?;

    // Only a party to the accepted commitment can open the dispute.
    if req.raised_by != commitment.accepted_by && req.raised_by != commitment.provider_id {
        return Err(DisputeError::NotACommitmentParty {
            party: req.raised_by.to_string(),
            commitment: req.commitment_ref.to_string(),
        });
    }

    // The commitment must be in a disputable settlement state.
    if !is_disputable_state(&commitment.settlement_state) {
        return Err(DisputeError::NotDisputable {
            commitment: req.commitment_ref.to_string(),
            state: commitment.settlement_state.clone(),
        });
    }

    let dispute_id = derive_dispute_id(req.commitment_ref, reason.as_str());

    // The same dispute request cannot be replayed.
    if state.disputes.contains_key(&dispute_id) {
        return Err(DisputeError::DuplicateDispute(dispute_id));
    }
    // Only one active dispute per commitment at a time.
    if state
        .disputes
        .values()
        .any(|d| d.commitment_ref == req.commitment_ref && d.status.is_active())
    {
        return Err(DisputeError::OpenDisputeExists(
            req.commitment_ref.to_string(),
        ));
    }

    let block = resolve_block(context, &mut state);
    let tx = resolve_tx(context, &format!("{dispute_id}:{block}:open"));
    let now = utc_now();

    let dispute = DisputeRecord {
        dispute_id: dispute_id.clone(),
        commitment_ref: req.commitment_ref.to_string(),
        raised_by: req.raised_by.to_string(),
        reason_code: reason,
        statement: req.statement,
        evidence_refs: req.evidence_refs,
        opened_at: now.clone(),
        response_deadline: req.response_deadline.to_string(),
        status: DisputeStatus::Open,
        resolution: None,
        updated_at: now.clone(),
        last_tx_hash: tx.clone(),
        last_block: block,
        last_source: context.source.clone(),
        last_network: context.network.clone(),
        last_chain_id: context.chain_id,
        last_contract: context.contract.clone(),
    };

    // Move the commitment into the disputed settlement state.
    if let Some(c) = state.commitments.get_mut(req.commitment_ref) {
        c.settlement_state = "disputed".to_string();
        c.updated_at = now.clone();
    }

    state.disputes.insert(dispute_id.clone(), dispute.clone());
    state.events.push(json!({
        "type": "dispute_opened",
        "dispute_id": dispute_id,
        "commitment_ref": req.commitment_ref,
        "raised_by": req.raised_by,
        "reason_code": reason.as_str(),
        "settlement_state": "disputed",
        "source": context.source,
        "network": context.network,
        "chain_id": context.chain_id,
        "contract": context.contract,
        "tx_hash": tx,
        "block": block,
        "timestamp": now
    }));
    save_state(&state)?;
    Ok(dispute)
}

/// Append supporting/rebuttal evidence to an active dispute. Rejected once the
/// response deadline has passed (evidence immutability).
pub fn append_dispute_evidence(
    dispute_id: &str,
    evidence: EvidenceRef,
) -> DisputeResult<DisputeRecord> {
    let mut state = load_state()?;
    let dispute = state
        .disputes
        .get(dispute_id)
        .cloned()
        .ok_or_else(|| DisputeError::UnknownDispute(dispute_id.to_string()))?;

    if dispute.status.is_terminal() {
        return Err(DisputeError::DisputeNotOpenForEvidence {
            dispute: dispute_id.to_string(),
            status: dispute.status.to_string(),
        });
    }

    let deadline = parse_rfc3339(&dispute.response_deadline, "response_deadline")?;
    if Utc::now() > deadline {
        return Err(DisputeError::EvidenceImmutable(dispute_id.to_string()));
    }

    let d = state
        .disputes
        .get_mut(dispute_id)
        .ok_or_else(|| DisputeError::UnknownDispute(dispute_id.to_string()))?;
    d.evidence_refs.push(evidence);
    d.updated_at = utc_now();
    let updated = d.clone();
    save_state(&state)?;
    Ok(updated)
}

pub fn get_dispute(dispute_id: &str) -> DisputeResult<DisputeRecord> {
    let state = load_state()?;
    state
        .disputes
        .get(dispute_id)
        .cloned()
        .ok_or_else(|| DisputeError::UnknownDispute(dispute_id.to_string()))
}

pub fn resolve_dispute(
    dispute_id: &str,
    decision: &str,
    resolved_by: &str,
    resolution_evidence_refs: Vec<EvidenceRef>,
) -> DisputeResult<DisputeRecord> {
    resolve_dispute_with_context(
        dispute_id,
        decision,
        resolved_by,
        resolution_evidence_refs,
        &WriteContext::local(),
    )
}

pub fn resolve_dispute_with_context(
    dispute_id: &str,
    decision: &str,
    resolved_by: &str,
    resolution_evidence_refs: Vec<EvidenceRef>,
    context: &WriteContext,
) -> DisputeResult<DisputeRecord> {
    let decision: ResolutionDecision =
        decision.parse().map_err(|_| DisputeError::InvalidDecision {
            value: decision.to_string(),
            allowed: ResolutionDecision::ALL
                .iter()
                .map(|d| d.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        })?;

    let mut state = load_state()?;
    let dispute = state
        .disputes
        .get(dispute_id)
        .cloned()
        .ok_or_else(|| DisputeError::UnknownDispute(dispute_id.to_string()))?;

    // A resolved dispute cannot be reopened/re-resolved under the same id.
    if dispute.status.is_terminal() {
        return Err(DisputeError::AlreadyResolved {
            dispute: dispute_id.to_string(),
            status: dispute.status.to_string(),
        });
    }

    let commitment = state
        .commitments
        .get(&dispute.commitment_ref)
        .cloned()
        .ok_or_else(|| DisputeError::UnknownCommitment(dispute.commitment_ref.clone()))?;

    // Only the assigned verifier or the named settlement authority can resolve.
    let is_verifier =
        !commitment.assigned_verifier.is_empty() && resolved_by == commitment.assigned_verifier;
    let is_settlement_authority =
        !commitment.settlement_authority.is_empty() && resolved_by == commitment.settlement_authority;
    if !is_verifier && !is_settlement_authority {
        return Err(DisputeError::Unauthorized {
            party: resolved_by.to_string(),
            dispute: dispute_id.to_string(),
            verifier: commitment.assigned_verifier.clone(),
            authority: commitment.settlement_authority.clone(),
        });
    }

    let block = resolve_block(context, &mut state);
    let tx = resolve_tx(context, &format!("{dispute_id}:{block}:resolve"));
    let now = utc_now();

    let resolution = DisputeResolution {
        resolution_id: format!("res_{}", &sha256_hex(&format!("{dispute_id}:{block}"))[..24]),
        dispute_id: dispute_id.to_string(),
        commitment_ref: dispute.commitment_ref.clone(),
        decision,
        resolved_by: resolved_by.to_string(),
        resolution_evidence_refs,
        resolved_at: now.clone(),
        settlement_state: decision,
    };

    let d = state
        .disputes
        .get_mut(dispute_id)
        .ok_or_else(|| DisputeError::UnknownDispute(dispute_id.to_string()))?;
    d.status = DisputeStatus::Resolved;
    d.resolution = Some(resolution.clone());
    d.updated_at = now.clone();
    d.last_tx_hash = tx.clone();
    d.last_block = block;
    d.last_source = context.source.clone();
    d.last_network = context.network.clone();
    d.last_chain_id = context.chain_id;
    d.last_contract = context.contract.clone();
    let updated = d.clone();

    if let Some(c) = state.commitments.get_mut(&dispute.commitment_ref) {
        c.settlement_state = decision.as_str().to_string();
        c.updated_at = now.clone();
    }

    state.events.push(json!({
        "type": "dispute_resolved",
        "dispute_id": dispute_id,
        "commitment_ref": dispute.commitment_ref,
        "decision": decision.as_str(),
        "resolved_by": resolved_by,
        "settlement_state": decision.as_str(),
        "source": context.source,
        "network": context.network,
        "chain_id": context.chain_id,
        "contract": context.contract,
        "tx_hash": tx,
        "block": block,
        "timestamp": now
    }));
    state.events.push(json!({
        "type": "settlement_recorded",
        "commitment_ref": dispute.commitment_ref,
        "state": decision.as_str(),
        "resolution_id": resolution.resolution_id,
        "timestamp": now
    }));
    save_state(&state)?;
    Ok(updated)
}

pub fn list_disputes() -> DisputeResult<Vec<DisputeRecord>> {
    let state = load_state()?;
    Ok(state.disputes.values().cloned().collect())
}

/// Resolve the effective block height: honour an on-chain context, otherwise
/// advance the local devnet head.
fn resolve_block(context: &WriteContext, state: &mut crate::models::RegistryState) -> u64 {
    if context.block > 0 {
        context.block
    } else {
        next_block(state)
    }
}

/// Resolve the effective transaction hash: honour an on-chain receipt, otherwise
/// derive a deterministic local hash from `seed`.
fn resolve_tx(context: &WriteContext, seed: &str) -> String {
    if context.tx_hash.is_empty() {
        tx_hash(seed)
    } else {
        context.tx_hash.clone()
    }
}
