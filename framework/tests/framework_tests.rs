use covenant_framework::chain::execute_registry_action;
use covenant_framework::config::NetworkDefinition;
use covenant_framework::conformance::run_conformance;
use covenant_framework::contracts::{open_dispute_action, register_action};
use covenant_framework::dispute::{
    append_dispute_evidence, get_dispute, open_dispute, record_commitment, resolve_dispute,
    OpenDispute, RecordCommitment,
};
use covenant_framework::domain::{DisputeStatus, ReasonCode, ResolutionDecision};
use covenant_framework::manifest::{sign_manifest, verify_manifest_signature};
use covenant_framework::models::EvidenceRef;
use covenant_framework::registry::{get_participant, register_participant, transition_participant};
use covenant_framework::store::{init_state, save_state, state_path};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static STATE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct StateGuard {
    lock: Option<MutexGuard<'static, ()>>,
    state_file: PathBuf,
    backup: Option<Vec<u8>>,
}

impl StateGuard {
    fn acquire() -> Self {
        let lock = STATE_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("state lock");
        let path = state_path();
        let backup = fs::read(&path).ok();
        Self {
            lock: Some(lock),
            state_file: path,
            backup,
        }
    }
}

impl Drop for StateGuard {
    fn drop(&mut self) {
        if let Some(parent) = self.state_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match &self.backup {
            Some(bytes) => {
                let _ = fs::write(&self.state_file, bytes);
            }
            None => {
                if self.state_file.exists() {
                    let _ = fs::remove_file(&self.state_file);
                }
            }
        }
        self.lock = None;
    }
}

struct TempDirGuard {
    dir: PathBuf,
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn make_temp_dir(prefix: &str) -> TempDirGuard {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("covenant_framework_{prefix}_{seed}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    TempDirGuard { dir }
}

fn framework_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

#[test]
fn provider_manifest_conformance_passes() {
    let result = run_conformance("provider-manifest.template.yaml").expect("conformance");
    assert!(
        result.passed,
        "expected provider template conformance to pass"
    );
    assert_eq!(result.participant_type, "provider");
}

#[test]
fn broker_manifest_conformance_passes() {
    let result = run_conformance("examples/broker.template.yaml").expect("conformance");
    assert!(
        result.passed,
        "expected broker template conformance to pass"
    );
    assert_eq!(result.participant_type, "broker");
}

#[test]
fn settlement_manifest_conformance_passes() {
    let result = run_conformance("examples/settlement-service.template.yaml").expect("conformance");
    assert!(
        result.passed,
        "expected settlement template conformance to pass"
    );
    assert_eq!(result.participant_type, "settlement_service");
}

#[test]
fn verifier_manifest_conformance_passes() {
    let result = run_conformance("examples/verifier.template.yaml").expect("conformance");
    assert!(
        result.passed,
        "expected verifier template conformance to pass"
    );
    assert_eq!(result.participant_type, "verifier");
}

#[test]
fn provider_manifest_fails_without_operational_controls() {
    let _guard = StateGuard::acquire();
    let tmp = make_temp_dir("provider_controls_invalid");
    let source = framework_root().join("provider-manifest.template.yaml");
    let raw = fs::read_to_string(&source).expect("read provider template");
    let mut doc: Value = serde_yaml::from_str(&raw).expect("parse provider template");
    if let Some(root) = doc.as_mapping_mut() {
        if let Some(policy) = root
            .get_mut(Value::String("provider_policy".to_string()))
            .and_then(Value::as_mapping_mut)
        {
            policy.remove(Value::String("incident_response_policy_ref".to_string()));
            policy.insert(
                Value::String("fulfillment_sla_seconds".to_string()),
                Value::Number(serde_yaml::Number::from(0)),
            );
        }
    }
    let manifest_path = tmp.dir.join("provider.controls.invalid.yaml");
    fs::write(
        &manifest_path,
        serde_yaml::to_string(&doc).expect("serialize provider manifest"),
    )
    .expect("write invalid provider manifest");

    let result =
        run_conformance(manifest_path.to_str().expect("manifest path")).expect("conformance");
    assert!(!result.passed, "invalid provider controls should fail");
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "provider_incident_policy_declared" && !c.passed),
        "incident policy check should fail"
    );
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "provider_fulfillment_sla_set" && !c.passed),
        "fulfillment SLA check should fail"
    );
}

#[test]
fn manifest_fails_when_conformance_profile_missing() {
    let _guard = StateGuard::acquire();
    let tmp = make_temp_dir("profile_missing");
    let source = framework_root().join("provider-manifest.template.yaml");
    let raw = fs::read_to_string(&source).expect("read provider template");
    let mut doc: Value = serde_yaml::from_str(&raw).expect("parse provider template");
    if let Some(root) = doc.as_mapping_mut() {
        root.remove(Value::String("conformance_profile".to_string()));
    }
    let manifest_path = tmp.dir.join("provider.profile.missing.yaml");
    fs::write(
        &manifest_path,
        serde_yaml::to_string(&doc).expect("serialize provider manifest"),
    )
    .expect("write invalid provider manifest");

    let result =
        run_conformance(manifest_path.to_str().expect("manifest path")).expect("conformance");
    assert!(!result.passed, "missing conformance profile should fail");
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "conformance_profile_supported" && !c.passed),
        "conformance profile check should fail"
    );
}

#[test]
fn manifest_fails_when_conformance_profile_is_unsupported() {
    let _guard = StateGuard::acquire();
    let tmp = make_temp_dir("profile_unsupported");
    let source = framework_root().join("provider-manifest.template.yaml");
    let raw = fs::read_to_string(&source).expect("read provider template");
    let mut doc: Value = serde_yaml::from_str(&raw).expect("parse provider template");
    if let Some(root) = doc.as_mapping_mut() {
        root.insert(
            Value::String("conformance_profile".to_string()),
            Value::String("onboarding-v0".to_string()),
        );
    }
    let manifest_path = tmp.dir.join("provider.profile.unsupported.yaml");
    fs::write(
        &manifest_path,
        serde_yaml::to_string(&doc).expect("serialize provider manifest"),
    )
    .expect("write invalid provider manifest");

    let result =
        run_conformance(manifest_path.to_str().expect("manifest path")).expect("conformance");
    assert!(
        !result.passed,
        "unsupported conformance profile should fail"
    );
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "conformance_profile_supported" && !c.passed),
        "conformance profile support check should fail"
    );
}

#[test]
fn broker_manifest_missing_policy_fails_actor_specific_checks() {
    let _guard = StateGuard::acquire();
    let tmp = make_temp_dir("broker_invalid");
    let source = framework_root()
        .join("examples")
        .join("broker.template.yaml");
    let raw = fs::read_to_string(&source).expect("read broker template");
    let mut doc: Value = serde_yaml::from_str(&raw).expect("parse broker template");
    if let Some(map) = doc.as_mapping_mut() {
        map.remove(Value::String("broker_policy".to_string()));
    }
    let manifest_path = tmp.dir.join("broker.invalid.yaml");
    fs::write(
        &manifest_path,
        serde_yaml::to_string(&doc).expect("serialize broker manifest"),
    )
    .expect("write invalid broker manifest");

    let result = run_conformance(manifest_path.to_str().expect("manifest path string"))
        .expect("conformance");
    assert!(
        !result.passed,
        "expected conformance to fail when broker_policy is missing"
    );
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "broker_routing_scope_declared" && !c.passed),
        "expected broker_routing_scope_declared check to fail"
    );
}

#[test]
fn broker_manifest_fails_without_conflict_controls() {
    let _guard = StateGuard::acquire();
    let tmp = make_temp_dir("broker_conflict_invalid");
    let source = framework_root()
        .join("examples")
        .join("broker.template.yaml");
    let raw = fs::read_to_string(&source).expect("read broker template");
    let mut doc: Value = serde_yaml::from_str(&raw).expect("parse broker template");
    if let Some(root) = doc.as_mapping_mut() {
        if let Some(policy) = root
            .get_mut(Value::String("broker_policy".to_string()))
            .and_then(Value::as_mapping_mut)
        {
            policy.remove(Value::String("conflict_of_interest_policy_ref".to_string()));
            policy.insert(
                Value::String("fairness_review_interval_days".to_string()),
                Value::Number(serde_yaml::Number::from(0)),
            );
        }
    }

    let manifest_path = tmp.dir.join("broker.conflict.invalid.yaml");
    fs::write(
        &manifest_path,
        serde_yaml::to_string(&doc).expect("serialize broker manifest"),
    )
    .expect("write invalid broker manifest");

    let result =
        run_conformance(manifest_path.to_str().expect("manifest path")).expect("conformance");
    assert!(!result.passed, "invalid broker controls should fail");
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "broker_conflict_policy_declared" && !c.passed),
        "conflict policy check should fail"
    );
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "broker_fairness_review_interval_set" && !c.passed),
        "fairness interval check should fail"
    );
}

#[test]
fn settlement_manifest_fails_without_replay_protection_and_finality() {
    let _guard = StateGuard::acquire();
    let tmp = make_temp_dir("settlement_invalid");
    let source = framework_root()
        .join("examples")
        .join("settlement-service.template.yaml");
    let raw = fs::read_to_string(&source).expect("read settlement template");
    let mut doc: Value = serde_yaml::from_str(&raw).expect("parse settlement template");
    if let Some(root) = doc.as_mapping_mut() {
        if let Some(policy) = root
            .get_mut(Value::String("settlement_policy".to_string()))
            .and_then(Value::as_mapping_mut)
        {
            policy.insert(
                Value::String("replay_protection".to_string()),
                Value::Bool(false),
            );
            policy.insert(
                Value::String("finality_wait_seconds".to_string()),
                Value::Number(serde_yaml::Number::from(0)),
            );
        }
    }

    let manifest_path = tmp.dir.join("settlement.invalid.yaml");
    fs::write(
        &manifest_path,
        serde_yaml::to_string(&doc).expect("serialize settlement manifest"),
    )
    .expect("write invalid settlement manifest");

    let result =
        run_conformance(manifest_path.to_str().expect("manifest path")).expect("conformance");
    assert!(!result.passed, "invalid settlement controls should fail");
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "settlement_replay_protection_enabled" && !c.passed),
        "replay protection check should fail"
    );
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "settlement_finality_wait_set" && !c.passed),
        "finality wait check should fail"
    );
}

#[test]
fn agent_manifest_conformance_passes_with_hardening_fields() {
    let result =
        run_conformance("examples/agent-service.template.yaml").expect("agent conformance");
    assert!(result.passed, "expected hardened agent manifest to pass");
    assert_eq!(result.participant_type, "agent_service");
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "agent_revocation_check_interval_set" && c.passed),
        "expected revocation check interval to be validated"
    );
}

#[test]
fn agent_manifest_fails_when_approval_and_revocation_controls_missing() {
    let _guard = StateGuard::acquire();
    let tmp = make_temp_dir("agent_invalid");
    let source = framework_root()
        .join("examples")
        .join("agent-service.template.yaml");
    let raw = fs::read_to_string(&source).expect("read agent template");
    let mut doc: Value = serde_yaml::from_str(&raw).expect("parse agent template");

    if let Some(root) = doc.as_mapping_mut() {
        if let Some(agent_policy) = root
            .get_mut(Value::String("agent_policy".to_string()))
            .and_then(Value::as_mapping_mut)
        {
            agent_policy.insert(
                Value::String("approval_gating_required".to_string()),
                Value::Bool(false),
            );
            agent_policy.remove(Value::String("grant_revocation_check_seconds".to_string()));
        }
    }

    let manifest_path = tmp.dir.join("agent.invalid.yaml");
    fs::write(
        &manifest_path,
        serde_yaml::to_string(&doc).expect("serialize agent manifest"),
    )
    .expect("write invalid agent manifest");

    let result =
        run_conformance(manifest_path.to_str().expect("manifest path")).expect("conformance");
    assert!(!result.passed, "invalid agent manifest should fail");
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "agent_approval_gating_enforced" && !c.passed),
        "approval gating check should fail"
    );
    assert!(
        result
            .checks
            .iter()
            .any(|c| c.name == "agent_revocation_check_interval_set" && !c.passed),
        "revocation interval check should fail"
    );
}

#[test]
fn local_onboarding_flow_reaches_active_and_rejects_invalid_transition() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");

    register_participant("provider-manifest.template.yaml", "0xabc123", 100.0).expect("register");
    transition_participant("provider.example.travel", "identity_verified", "test")
        .expect("identity transition");
    transition_participant("provider.example.travel", "conformance_passed", "test")
        .expect("conformance transition");
    transition_participant("provider.example.travel", "probation", "test")
        .expect("probation transition");
    transition_participant("provider.example.travel", "active", "test").expect("active transition");

    let participant = get_participant("provider.example.travel").expect("participant");
    assert_eq!(participant.state, "active");

    let invalid = transition_participant("provider.example.travel", "requested", "invalid");
    assert!(invalid.is_err(), "invalid transition should be rejected");
}

#[test]
fn chain_mode_requires_raw_tx_before_network_calls() {
    let network = NetworkDefinition {
        chain_id: 84532,
        rpc_url: "http://127.0.0.1:1".to_string(),
        registry_contract: "0x0000000000000000000000000000000000000000".to_string(),
        explorer_url: "https://example.com".to_string(),
    };
    let action = register_action("provider.example.travel", "sha256:abc", "0xabc123", 100.0);
    let result = execute_registry_action("base_sepolia", &network, &action, &None);
    assert!(result.is_err(), "missing raw tx should fail");
    let message = result.err().unwrap_or_default();
    assert!(
        message.contains("requires --raw-tx"),
        "expected raw-tx requirement error, got: {message}"
    );
}

#[test]
fn detached_signature_roundtrip_succeeds() {
    let _guard = StateGuard::acquire();
    let tmp = make_temp_dir("signature");
    let source = framework_root().join("provider-manifest.template.yaml");
    let raw = fs::read_to_string(&source).expect("read provider manifest template");
    let mut doc: Value = serde_yaml::from_str(&raw).expect("parse provider manifest");
    if let Some(map) = doc.as_mapping_mut() {
        if let Some(sig) = map.get_mut(Value::String("signature".to_string())) {
            if let Some(sig_map) = sig.as_mapping_mut() {
                sig_map.insert(
                    Value::String("file".to_string()),
                    Value::String("manifest.sig".to_string()),
                );
            }
        }
    }
    let manifest_path = tmp.dir.join("manifest.yaml");
    fs::write(
        &manifest_path,
        serde_yaml::to_string(&doc).expect("serialize provider manifest"),
    )
    .expect("write test manifest");

    let manifest_path_str = manifest_path.to_str().expect("manifest path");
    let signature_path = sign_manifest(manifest_path_str).expect("sign manifest");
    verify_manifest_signature(manifest_path_str).expect("verify signature");
    assert!(
        Path::new(&signature_path).exists(),
        "signature file should be created"
    );
}

fn sample_evidence() -> Vec<EvidenceRef> {
    vec![
        EvidenceRef {
            r#type: "accepted_offer".to_string(),
            digest: format!("sha256:{}", "a".repeat(64)),
        },
        EvidenceRef {
            r#type: "provider_receipt".to_string(),
            digest: format!("sha256:{}", "b".repeat(64)),
        },
    ]
}

fn resolution_evidence() -> Vec<EvidenceRef> {
    vec![EvidenceRef {
        r#type: "verifier_report".to_string(),
        digest: format!("sha256:{}", "c".repeat(64)),
    }]
}

const FUTURE_DEADLINE: &str = "2999-01-01T00:00:00Z";
const PAST_DEADLINE: &str = "2000-01-01T00:00:00Z";

fn seed_commitment(commitment_ref: &str) {
    record_commitment(&RecordCommitment {
        commitment_ref,
        objective_ref: "obj_test_001",
        offer_id: "ofr_test_001",
        accepted_by: "agent.test.orchestrator",
        provider_id: "provider.test.charlie",
        assigned_verifier: "verifier.test",
        settlement_authority: "settlement.test.service",
        settlement_state: "fulfilled",
    })
    .expect("record commitment");
}

#[test]
fn dispute_happy_path_record_open_resolve() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");

    seed_commitment("cmt_happy_001");
    let dispute = open_dispute(OpenDispute {
        commitment_ref: "cmt_happy_001",
        raised_by: "agent.test.orchestrator",
        reason_code: "terms_mismatch",
        statement: Some("cabin mismatch".to_string()),
        evidence_refs: sample_evidence(),
        response_deadline: FUTURE_DEADLINE,
    })
    .expect("open dispute");
    assert_eq!(dispute.status, DisputeStatus::Open);
    assert_eq!(dispute.reason_code, ReasonCode::TermsMismatch);

    let resolved = resolve_dispute(
        &dispute.dispute_id,
        "refunded",
        "verifier.test",
        resolution_evidence(),
    )
    .expect("resolve dispute");
    assert_eq!(resolved.status, DisputeStatus::Resolved);
    let resolution = resolved.resolution.expect("resolution present");
    assert_eq!(resolution.decision, ResolutionDecision::Refunded);
    assert_eq!(resolution.settlement_state, ResolutionDecision::Refunded);
}

#[test]
fn dispute_open_rejected_for_non_party() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");
    seed_commitment("cmt_party_001");

    let result = open_dispute(OpenDispute {
        commitment_ref: "cmt_party_001",
        raised_by: "stranger.outsider",
        reason_code: "terms_mismatch",
        statement: None,
        evidence_refs: sample_evidence(),
        response_deadline: FUTURE_DEADLINE,
    });
    assert!(result.is_err(), "non-party open should be rejected");
    assert!(result.unwrap_err().to_string().contains("not a party"));
}

#[test]
fn dispute_open_rejected_when_not_disputable() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");
    record_commitment(&RecordCommitment {
        commitment_ref: "cmt_state_001",
        objective_ref: "obj",
        offer_id: "ofr",
        accepted_by: "agent.test.orchestrator",
        provider_id: "provider.test.charlie",
        assigned_verifier: "verifier.test",
        settlement_authority: "settlement.test.service",
        settlement_state: "refunded",
    })
    .expect("record commitment");

    let result = open_dispute(OpenDispute {
        commitment_ref: "cmt_state_001",
        raised_by: "agent.test.orchestrator",
        reason_code: "terms_mismatch",
        statement: None,
        evidence_refs: sample_evidence(),
        response_deadline: FUTURE_DEADLINE,
    });
    assert!(result.is_err(), "non-disputable state should be rejected");
    assert!(result.unwrap_err().to_string().contains("not disputable"));
}

#[test]
fn dispute_open_rejected_for_unknown_reason_code() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");
    seed_commitment("cmt_reason_001");

    let result = open_dispute(OpenDispute {
        commitment_ref: "cmt_reason_001",
        raised_by: "agent.test.orchestrator",
        reason_code: "made_up_reason",
        statement: None,
        evidence_refs: sample_evidence(),
        response_deadline: FUTURE_DEADLINE,
    });
    assert!(result.is_err(), "unknown reason code should be rejected");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("unknown reason_code"));
}

#[test]
fn dispute_replay_same_request_rejected() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");
    seed_commitment("cmt_replay_001");

    open_dispute(OpenDispute {
        commitment_ref: "cmt_replay_001",
        raised_by: "agent.test.orchestrator",
        reason_code: "terms_mismatch",
        statement: None,
        evidence_refs: sample_evidence(),
        response_deadline: FUTURE_DEADLINE,
    })
    .expect("first open");

    let replay = open_dispute(OpenDispute {
        commitment_ref: "cmt_replay_001",
        raised_by: "agent.test.orchestrator",
        reason_code: "terms_mismatch",
        statement: None,
        evidence_refs: sample_evidence(),
        response_deadline: FUTURE_DEADLINE,
    });
    assert!(replay.is_err(), "replayed dispute should be rejected");
    let message = replay.unwrap_err().to_string();
    assert!(
        message.contains("replay") || message.contains("open dispute already exists"),
        "expected replay error, got: {message}"
    );
}

#[test]
fn dispute_resolve_rejected_for_unauthorized() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");
    seed_commitment("cmt_auth_001");
    let dispute = open_dispute(OpenDispute {
        commitment_ref: "cmt_auth_001",
        raised_by: "agent.test.orchestrator",
        reason_code: "terms_mismatch",
        statement: None,
        evidence_refs: sample_evidence(),
        response_deadline: FUTURE_DEADLINE,
    })
    .expect("open dispute");

    let result = resolve_dispute(
        &dispute.dispute_id,
        "refunded",
        "stranger.outsider",
        resolution_evidence(),
    );
    assert!(result.is_err(), "unauthorized resolve should be rejected");
    assert!(result.unwrap_err().to_string().contains("not authorized"));
}

#[test]
fn dispute_resolve_allows_settlement_authority() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");
    seed_commitment("cmt_settle_001");
    let dispute = open_dispute(OpenDispute {
        commitment_ref: "cmt_settle_001",
        raised_by: "agent.test.orchestrator",
        reason_code: "terms_mismatch",
        statement: None,
        evidence_refs: sample_evidence(),
        response_deadline: FUTURE_DEADLINE,
    })
    .expect("open dispute");

    let resolved = resolve_dispute(
        &dispute.dispute_id,
        "failed",
        "settlement.test.service",
        resolution_evidence(),
    )
    .expect("settlement authority resolve");
    assert_eq!(resolved.status, DisputeStatus::Resolved);
}

#[test]
fn dispute_cannot_be_reresolved() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");
    seed_commitment("cmt_reres_001");
    let dispute = open_dispute(OpenDispute {
        commitment_ref: "cmt_reres_001",
        raised_by: "agent.test.orchestrator",
        reason_code: "terms_mismatch",
        statement: None,
        evidence_refs: sample_evidence(),
        response_deadline: FUTURE_DEADLINE,
    })
    .expect("open dispute");
    resolve_dispute(
        &dispute.dispute_id,
        "refunded",
        "verifier.test",
        resolution_evidence(),
    )
    .expect("first resolve");

    let again = resolve_dispute(
        &dispute.dispute_id,
        "fulfilled",
        "verifier.test",
        resolution_evidence(),
    );
    assert!(again.is_err(), "re-resolution should be rejected");
    assert!(again.unwrap_err().to_string().contains("already resolved"));
}

#[test]
fn dispute_evidence_immutable_after_deadline() {
    let _guard = StateGuard::acquire();
    save_state(&init_state()).expect("reset state");
    seed_commitment("cmt_deadline_001");
    let dispute = open_dispute(OpenDispute {
        commitment_ref: "cmt_deadline_001",
        raised_by: "agent.test.orchestrator",
        reason_code: "terms_mismatch",
        statement: None,
        evidence_refs: sample_evidence(),
        response_deadline: PAST_DEADLINE,
    })
    .expect("open dispute");

    let extra = EvidenceRef {
        r#type: "late_addendum".to_string(),
        digest: format!("sha256:{}", "d".repeat(64)),
    };
    let result = append_dispute_evidence(&dispute.dispute_id, extra);
    assert!(
        result.is_err(),
        "evidence append after deadline should be rejected"
    );
    assert!(result.unwrap_err().to_string().contains("immutable"));

    let stored = get_dispute(&dispute.dispute_id).expect("dispute");
    assert_eq!(stored.evidence_refs.len(), 2, "evidence set must be frozen");
}

#[test]
fn dispute_chain_action_requires_raw_tx() {
    let network = NetworkDefinition {
        chain_id: 84532,
        rpc_url: "http://127.0.0.1:1".to_string(),
        registry_contract: "0x0000000000000000000000000000000000000000".to_string(),
        explorer_url: "https://example.com".to_string(),
    };
    let action = open_dispute_action(
        "dsp_test",
        "cmt_test",
        "sha256:root",
        "2999-01-01T00:00:00Z",
    );
    let result = execute_registry_action("base_sepolia", &network, &action, &None);
    assert!(result.is_err(), "missing raw tx should fail");
    assert!(result
        .err()
        .unwrap_or_default()
        .contains("requires --raw-tx"));
}
