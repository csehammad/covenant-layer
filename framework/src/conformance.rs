use crate::manifest::{load_manifest, validate_manifest};
use serde::Serialize;
use serde_yaml::{Mapping, Value};

const REQUIRED_CONFORMANCE_PROFILE: &str = "onboarding-v1";

#[derive(Debug, Serialize)]
pub struct ConformanceCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct ConformanceResult {
    pub passed: bool,
    pub participant_id: String,
    pub participant_type: String,
    pub checks: Vec<ConformanceCheck>,
    pub errors: Vec<String>,
    pub recommended_state: String,
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_string()))
}

fn mapping_has_nonempty_seq(map: &Mapping, parent: &str, child: &str) -> bool {
    mapping_get(map, parent)
        .and_then(Value::as_mapping)
        .and_then(|m| mapping_get(m, child))
        .and_then(Value::as_sequence)
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

fn mapping_has_bool(map: &Mapping, parent: &str, child: &str, expected: bool) -> bool {
    mapping_get(map, parent)
        .and_then(Value::as_mapping)
        .and_then(|m| mapping_get(m, child))
        .and_then(Value::as_bool)
        .map(|v| v == expected)
        .unwrap_or(false)
}

fn mapping_has_string(map: &Mapping, parent: &str, child: &str) -> bool {
    mapping_get(map, parent)
        .and_then(Value::as_mapping)
        .and_then(|m| mapping_get(m, child))
        .and_then(Value::as_str)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

fn mapping_has_positive_u64(map: &Mapping, parent: &str, child: &str) -> bool {
    mapping_get(map, parent)
        .and_then(Value::as_mapping)
        .and_then(|m| mapping_get(m, child))
        .and_then(Value::as_u64)
        .map(|v| v > 0)
        .unwrap_or(false)
}

fn mapping_has_root_string_value(map: &Mapping, key: &str, expected: &str) -> bool {
    mapping_get(map, key)
        .and_then(Value::as_str)
        .map(|v| v == expected)
        .unwrap_or(false)
}

pub fn run_conformance(manifest_path: &str) -> Result<ConformanceResult, String> {
    let manifest = load_manifest(manifest_path)?;
    let errors = validate_manifest(&manifest);
    let participant_id = manifest
        .as_mapping()
        .and_then(|m| m.get(serde_yaml::Value::String("participant_id".into())))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let participant_type = manifest
        .as_mapping()
        .and_then(|m| m.get(serde_yaml::Value::String("participant_type".into())))
        .and_then(serde_yaml::Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let mut checks: Vec<ConformanceCheck> = vec![];
    let m = manifest.as_mapping();

    let has_identity = m
        .and_then(|x| x.get(serde_yaml::Value::String("identity".into())))
        .is_some();
    checks.push(ConformanceCheck {
        name: "identity_present".to_string(),
        passed: has_identity,
        detail: "identity block required".to_string(),
    });

    let has_signing_key = m
        .and_then(|x| x.get(serde_yaml::Value::String("keys".into())))
        .and_then(serde_yaml::Value::as_mapping)
        .and_then(|k| k.get(serde_yaml::Value::String("signing".into())))
        .and_then(serde_yaml::Value::as_sequence)
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);
    checks.push(ConformanceCheck {
        name: "signing_keys_present".to_string(),
        passed: has_signing_key,
        detail: "at least one signing key".to_string(),
    });

    let has_services = m
        .and_then(|x| x.get(serde_yaml::Value::String("service_interfaces".into())))
        .is_some();
    checks.push(ConformanceCheck {
        name: "service_interfaces_declared".to_string(),
        passed: has_services,
        detail: "service interfaces required".to_string(),
    });

    checks.push(ConformanceCheck {
        name: "manifest_validation".to_string(),
        passed: errors.is_empty(),
        detail: "required fields and state validity".to_string(),
    });

    let initial_state_requested = m
        .and_then(|x| x.get(serde_yaml::Value::String("admission_status".into())))
        .and_then(serde_yaml::Value::as_mapping)
        .and_then(|a| a.get(serde_yaml::Value::String("state".into())))
        .and_then(serde_yaml::Value::as_str)
        .map(|v| v == "requested")
        .unwrap_or(false);
    checks.push(ConformanceCheck {
        name: "admission_state_initialized".to_string(),
        passed: initial_state_requested,
        detail: "initial state should be requested".to_string(),
    });

    if let Some(map) = m {
        checks.push(ConformanceCheck {
            name: "conformance_profile_supported".to_string(),
            passed: mapping_has_root_string_value(
                map,
                "conformance_profile",
                REQUIRED_CONFORMANCE_PROFILE,
            ),
            detail: format!("manifest must set conformance_profile={REQUIRED_CONFORMANCE_PROFILE}"),
        });

        match participant_type.as_str() {
            "provider" => {
                checks.push(ConformanceCheck {
                    name: "provider_objective_types_declared".to_string(),
                    passed: mapping_has_nonempty_seq(map, "capabilities", "objective_types"),
                    detail: "provider must declare capability objective_types".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "provider_evidence_types_declared".to_string(),
                    passed: mapping_has_nonempty_seq(map, "evidence_profile", "required_types"),
                    detail: "provider must declare evidence_profile.required_types".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "provider_incident_policy_declared".to_string(),
                    passed: mapping_has_string(
                        map,
                        "provider_policy",
                        "incident_response_policy_ref",
                    ),
                    detail: "provider must declare incident response policy reference".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "provider_data_protection_policy_declared".to_string(),
                    passed: mapping_has_string(
                        map,
                        "provider_policy",
                        "data_protection_policy_ref",
                    ),
                    detail: "provider must declare data protection policy reference".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "provider_fulfillment_sla_set".to_string(),
                    passed: mapping_has_positive_u64(
                        map,
                        "provider_policy",
                        "fulfillment_sla_seconds",
                    ),
                    detail: "provider must set positive fulfillment SLA seconds".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "provider_idempotency_window_set".to_string(),
                    passed: mapping_has_positive_u64(
                        map,
                        "provider_policy",
                        "idempotency_window_seconds",
                    ),
                    detail: "provider must set positive idempotency window seconds".to_string(),
                });
            }
            "broker" => {
                checks.push(ConformanceCheck {
                    name: "broker_routing_scope_declared".to_string(),
                    passed: mapping_has_nonempty_seq(map, "broker_policy", "routing_scope"),
                    detail: "broker must declare broker_policy.routing_scope".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "broker_provenance_policy_declared".to_string(),
                    passed: mapping_has_string(map, "broker_policy", "offer_provenance_policy_ref"),
                    detail: "broker must declare offer provenance policy reference".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "broker_conflict_policy_declared".to_string(),
                    passed: mapping_has_string(
                        map,
                        "broker_policy",
                        "conflict_of_interest_policy_ref",
                    ),
                    detail: "broker must declare conflict-of-interest policy reference".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "broker_audit_retention_set".to_string(),
                    passed: mapping_has_positive_u64(
                        map,
                        "broker_policy",
                        "routing_audit_log_retention_days",
                    ),
                    detail: "broker must set positive routing audit retention days".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "broker_fairness_review_interval_set".to_string(),
                    passed: mapping_has_positive_u64(
                        map,
                        "broker_policy",
                        "fairness_review_interval_days",
                    ),
                    detail: "broker must set positive fairness review interval days".to_string(),
                });
            }
            "settlement_service" => {
                checks.push(ConformanceCheck {
                    name: "settlement_supported_states_declared".to_string(),
                    passed: mapping_has_nonempty_seq(map, "settlement_policy", "supported_states"),
                    detail: "settlement_service must declare settlement_policy.supported_states"
                        .to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "settlement_dispute_policy_declared".to_string(),
                    passed: mapping_has_string(
                        map,
                        "settlement_policy",
                        "dispute_transition_policy_ref",
                    ),
                    detail: "settlement_service must declare dispute transition policy reference"
                        .to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "settlement_evidence_integrity_policy_declared".to_string(),
                    passed: mapping_has_string(
                        map,
                        "settlement_policy",
                        "evidence_link_integrity_ref",
                    ),
                    detail: "settlement_service must declare evidence integrity policy reference"
                        .to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "settlement_finality_wait_set".to_string(),
                    passed: mapping_has_positive_u64(
                        map,
                        "settlement_policy",
                        "finality_wait_seconds",
                    ),
                    detail: "settlement_service must set positive finality wait seconds"
                        .to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "settlement_replay_protection_enabled".to_string(),
                    passed: mapping_has_bool(map, "settlement_policy", "replay_protection", true),
                    detail: "settlement_service must enable replay protection".to_string(),
                });
            }
            "verifier" => {
                checks.push(ConformanceCheck {
                    name: "verifier_domains_declared".to_string(),
                    passed: mapping_has_nonempty_seq(
                        map,
                        "verification_policy",
                        "verification_domains",
                    ),
                    detail: "verifier must declare verification domains".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "verifier_replay_protection_enabled".to_string(),
                    passed: mapping_has_bool(map, "verification_policy", "replay_protection", true),
                    detail: "verifier must enable replay protection".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "verifier_sla_set".to_string(),
                    passed: mapping_has_positive_u64(
                        map,
                        "verification_policy",
                        "verification_sla_seconds",
                    ),
                    detail: "verifier must set positive verification SLA seconds".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "verifier_decision_policy_declared".to_string(),
                    passed: mapping_has_string(map, "verification_policy", "decision_policy_ref"),
                    detail: "verifier must declare decision policy reference".to_string(),
                });
            }
            "agent_service" => {
                checks.push(ConformanceCheck {
                    name: "agent_authority_policy_declared".to_string(),
                    passed: mapping_has_string(
                        map,
                        "agent_policy",
                        "authority_handling_policy_ref",
                    ),
                    detail: "agent_service must declare authority handling policy".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "agent_approval_gating_enforced".to_string(),
                    passed: mapping_has_bool(map, "agent_policy", "approval_gating_required", true),
                    detail: "agent_service must enforce approval gating".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "agent_authority_proof_method_declared".to_string(),
                    passed: mapping_has_string(map, "agent_policy", "authority_proof_method"),
                    detail: "agent_service must declare authority proof method".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "agent_approval_reference_format_declared".to_string(),
                    passed: mapping_has_string(map, "agent_policy", "approval_reference_format"),
                    detail: "agent_service must declare approval reference format".to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "agent_revocation_check_interval_set".to_string(),
                    passed: mapping_has_positive_u64(
                        map,
                        "agent_policy",
                        "grant_revocation_check_seconds",
                    ),
                    detail: "agent_service must set positive grant revocation check interval"
                        .to_string(),
                });
                checks.push(ConformanceCheck {
                    name: "agent_max_authority_ttl_set".to_string(),
                    passed: mapping_has_positive_u64(
                        map,
                        "agent_policy",
                        "max_authority_ttl_seconds",
                    ),
                    detail: "agent_service must set positive max authority TTL".to_string(),
                });
            }
            _ => {
                checks.push(ConformanceCheck {
                    name: "participant_type_known".to_string(),
                    passed: false,
                    detail: "participant_type must be one of provider|broker|settlement_service|verifier|agent_service".to_string(),
                });
            }
        }
    }

    let passed = checks.iter().all(|c| c.passed);
    Ok(ConformanceResult {
        passed,
        participant_id,
        participant_type,
        checks,
        errors,
        recommended_state: if passed {
            "conformance_passed".to_string()
        } else {
            "restricted".to_string()
        },
    })
}
