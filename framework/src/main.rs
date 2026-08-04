use clap::{Parser, Subcommand, ValueEnum};
use covenant_framework::chain::{execute_registry_action, fetch_chain_context};
use covenant_framework::config::resolve_network;
use covenant_framework::conformance::run_conformance;
use covenant_framework::contracts::{
    attestation_action, open_dispute_action, register_action, resolve_dispute_action,
    transition_action,
};
use covenant_framework::dispute::{
    derive_dispute_id, get_dispute, open_dispute, open_dispute_with_context, record_commitment,
    record_commitment_with_context, resolve_dispute, resolve_dispute_with_context, OpenDispute,
    RecordCommitment, WriteContext as DisputeWriteContext,
};
use covenant_framework::manifest::{self, sign_manifest, verify_manifest_signature};
use covenant_framework::models::EvidenceRef;
use covenant_framework::registry::{
    add_attestation, add_attestation_with_context, get_participant, list_participants,
    register_participant, register_participant_with_context, transition_participant,
    transition_participant_with_context, WriteContext,
};
use covenant_framework::store::{init_state, save_state};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, ValueEnum)]
enum Mode {
    Local,
    Chain,
}

#[derive(Clone, Debug, ValueEnum)]
enum StatusSource {
    Local,
    Chain,
}

#[derive(Parser)]
#[command(name = "covenant-framework")]
#[command(about = "Rust framework runtime for onboarding + registry flow", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Register {
        #[arg(long)]
        manifest: String,
        #[arg(long)]
        owner: String,
        #[arg(long, default_value_t = 0.0)]
        stake: f64,
        #[arg(long, value_enum, default_value_t = Mode::Local)]
        mode: Mode,
        #[arg(long)]
        network: Option<String>,
        #[arg(long = "rpc-url")]
        rpc_url: Option<String>,
        #[arg(long)]
        contract: Option<String>,
        #[arg(long = "raw-tx")]
        raw_tx: Option<String>,
    },
    Conformance {
        #[arg(long)]
        manifest: String,
        #[arg(long, default_value_t = false)]
        attach: bool,
        #[arg(long, value_enum, default_value_t = Mode::Local)]
        mode: Mode,
        #[arg(long)]
        network: Option<String>,
        #[arg(long = "rpc-url")]
        rpc_url: Option<String>,
        #[arg(long)]
        contract: Option<String>,
        #[arg(long = "raw-tx")]
        raw_tx: Option<String>,
    },
    Transition {
        #[arg(long = "participant-id")]
        participant_id: String,
        #[arg(long = "to-state")]
        to_state: String,
        #[arg(long, default_value = "manual transition")]
        reason: String,
        #[arg(long, value_enum, default_value_t = Mode::Local)]
        mode: Mode,
        #[arg(long)]
        network: Option<String>,
        #[arg(long = "rpc-url")]
        rpc_url: Option<String>,
        #[arg(long)]
        contract: Option<String>,
        #[arg(long = "raw-tx")]
        raw_tx: Option<String>,
    },
    Status {
        #[arg(long = "participant-id")]
        participant_id: Option<String>,
        #[arg(long, value_enum, default_value_t = StatusSource::Local)]
        source: StatusSource,
        #[arg(long)]
        network: Option<String>,
        #[arg(long = "rpc-url")]
        rpc_url: Option<String>,
        #[arg(long)]
        contract: Option<String>,
    },
    E2e {
        #[arg(long)]
        manifest: String,
        #[arg(long)]
        owner: String,
        #[arg(long, default_value_t = 0.0)]
        stake: f64,
        #[arg(long, value_enum, default_value_t = Mode::Local)]
        mode: Mode,
        #[arg(long)]
        network: Option<String>,
        #[arg(long = "rpc-url")]
        rpc_url: Option<String>,
        #[arg(long)]
        contract: Option<String>,
        #[arg(long = "register-raw-tx")]
        register_raw_tx: Option<String>,
        #[arg(long = "transition-raw-tx")]
        transition_raw_tx: Option<String>,
    },
    Sign {
        #[arg(long)]
        manifest: String,
    },
    VerifySignature {
        #[arg(long)]
        manifest: String,
    },
    ValidateE2e {
        #[arg(long)]
        manifest: String,
        #[arg(long)]
        owner: String,
    },
    /// Record (seed) an accepted commitment used to anchor dispute enforcement.
    CommitmentRecord {
        #[arg(long = "commitment-id")]
        commitment_id: String,
        #[arg(long = "objective-ref", default_value = "")]
        objective_ref: String,
        #[arg(long = "offer-id", default_value = "")]
        offer_id: String,
        #[arg(long = "accepted-by")]
        accepted_by: String,
        #[arg(long = "provider-id")]
        provider_id: String,
        #[arg(long = "assigned-verifier", default_value = "")]
        assigned_verifier: String,
        #[arg(long = "settlement-authority", default_value = "")]
        settlement_authority: String,
        #[arg(long = "settlement-state", default_value = "fulfilled")]
        settlement_state: String,
        #[arg(long, value_enum, default_value_t = Mode::Local)]
        mode: Mode,
        #[arg(long)]
        network: Option<String>,
        #[arg(long = "rpc-url")]
        rpc_url: Option<String>,
        #[arg(long)]
        contract: Option<String>,
        #[arg(long = "raw-tx")]
        raw_tx: Option<String>,
    },
    DisputeOpen {
        #[arg(long = "commitment-id")]
        commitment_id: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        evidence: String,
        #[arg(long = "raised-by")]
        raised_by: String,
        #[arg(long = "response-deadline")]
        response_deadline: String,
        #[arg(long, value_enum, default_value_t = Mode::Local)]
        mode: Mode,
        #[arg(long)]
        network: Option<String>,
        #[arg(long = "rpc-url")]
        rpc_url: Option<String>,
        #[arg(long)]
        contract: Option<String>,
        #[arg(long = "raw-tx")]
        raw_tx: Option<String>,
    },
    DisputeStatus {
        #[arg(long = "dispute-id")]
        dispute_id: String,
    },
    DisputeResolve {
        #[arg(long = "dispute-id")]
        dispute_id: String,
        #[arg(long)]
        decision: String,
        #[arg(long)]
        resolution: String,
        #[arg(long = "resolved-by")]
        resolved_by: String,
        #[arg(long, value_enum, default_value_t = Mode::Local)]
        mode: Mode,
        #[arg(long)]
        network: Option<String>,
        #[arg(long = "rpc-url")]
        rpc_url: Option<String>,
        #[arg(long)]
        contract: Option<String>,
        #[arg(long = "raw-tx")]
        raw_tx: Option<String>,
    },
}

#[derive(serde::Deserialize, Default)]
struct EvidenceFile {
    #[serde(default)]
    evidence_refs: Vec<EvidenceRefInput>,
    #[serde(default)]
    resolution_evidence_refs: Vec<EvidenceRefInput>,
    #[serde(default)]
    statement: Option<String>,
}

#[derive(serde::Deserialize)]
struct EvidenceRefInput {
    r#type: String,
    digest: String,
}

fn read_evidence_file(path: &str) -> Result<EvidenceFile, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read evidence file {path}: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Failed to parse evidence file {path}: {e}"))
}

fn to_evidence_refs(items: Vec<EvidenceRefInput>) -> Vec<EvidenceRef> {
    items
        .into_iter()
        .map(|i| EvidenceRef {
            r#type: i.r#type,
            digest: i.digest,
        })
        .collect()
}

/// Deterministic evidence root over the ordered evidence digests, used as the
/// on-chain commitment/dispute evidence anchor (off-chain payloads stay off-chain).
fn evidence_root(refs: &[EvidenceRef]) -> String {
    let mut hasher = Sha256::new();
    for r in refs {
        hasher.update(r.r#type.as_bytes());
        hasher.update(b":");
        hasher.update(r.digest.as_bytes());
        hasher.update(b"\n");
    }
    format!("sha256:{:x}", hasher.finalize())
}

fn print_json<T: serde::Serialize>(value: &T) {
    let output = serde_json::to_string_pretty(value).expect("serializable output");
    println!("{output}");
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            save_state(&init_state())?;
            print_json(
                &serde_json::json!({"ok": true, "message": "Initialized framework state store"}),
            );
        }
        Commands::Register {
            manifest,
            owner,
            stake,
            mode,
            network,
            rpc_url,
            contract,
            raw_tx,
        } => {
            let record = match mode {
                Mode::Local => register_participant(&manifest, &owner, stake)?,
                Mode::Chain => {
                    verify_manifest_signature(&manifest)?;
                    let (network_name, network_def) =
                        resolve_network(&network, &rpc_url, &contract)?;
                    let action = register_action(
                        &manifest::manifest_field_string(
                            &manifest::load_manifest(&manifest)?,
                            "participant_id",
                        )?,
                        &manifest::compute_manifest_hash(&manifest)?,
                        &owner,
                        stake,
                    );
                    let artifact =
                        execute_registry_action(&network_name, &network_def, &action, &raw_tx)?;
                    let ctx = WriteContext {
                        source: artifact.source,
                        tx_hash: artifact.tx_hash,
                        block: artifact.block_number,
                        network: Some(artifact.network),
                        chain_id: Some(artifact.chain_id),
                        contract: Some(artifact.contract),
                    };
                    register_participant_with_context(&manifest, &owner, stake, &ctx)?
                }
            };
            print_json(&record);
        }
        Commands::Conformance {
            manifest,
            attach,
            mode,
            network,
            rpc_url,
            contract,
            raw_tx,
        } => {
            let result = run_conformance(&manifest)?;
            print_json(&result);
            if attach && result.participant_id != "unknown" {
                let details = serde_json::to_value(&result).map_err(|e| e.to_string())?;
                let att = match mode {
                    Mode::Local => {
                        add_attestation(&result.participant_id, "conformance_test", details)?
                    }
                    Mode::Chain => {
                        let (network_name, network_def) =
                            resolve_network(&network, &rpc_url, &contract)?;
                        let action = attestation_action(&result.participant_id, "conformance_test");
                        let artifact =
                            execute_registry_action(&network_name, &network_def, &action, &raw_tx)?;
                        let ctx = WriteContext {
                            source: artifact.source,
                            tx_hash: artifact.tx_hash,
                            block: artifact.block_number,
                            network: Some(artifact.network),
                            chain_id: Some(artifact.chain_id),
                            contract: Some(artifact.contract),
                        };
                        add_attestation_with_context(
                            &result.participant_id,
                            "conformance_test",
                            serde_json::to_value(&result).map_err(|e| e.to_string())?,
                            &ctx,
                        )?
                    }
                };
                print_json(&serde_json::json!({ "attached_attestation": att }));
            }
        }
        Commands::Transition {
            participant_id,
            to_state,
            reason,
            mode,
            network,
            rpc_url,
            contract,
            raw_tx,
        } => {
            let record = match mode {
                Mode::Local => transition_participant(&participant_id, &to_state, &reason)?,
                Mode::Chain => {
                    let current = get_participant(&participant_id)?;
                    let (network_name, network_def) =
                        resolve_network(&network, &rpc_url, &contract)?;
                    let action =
                        transition_action(&participant_id, &current.state, &to_state, &reason);
                    let artifact =
                        execute_registry_action(&network_name, &network_def, &action, &raw_tx)?;
                    let ctx = WriteContext {
                        source: artifact.source,
                        tx_hash: artifact.tx_hash,
                        block: artifact.block_number,
                        network: Some(artifact.network),
                        chain_id: Some(artifact.chain_id),
                        contract: Some(artifact.contract),
                    };
                    transition_participant_with_context(&participant_id, &to_state, &reason, &ctx)?
                }
            };
            print_json(&record);
        }
        Commands::Status {
            participant_id,
            source,
            network,
            rpc_url,
            contract,
        } => {
            let mut participants = if let Some(id) = participant_id {
                vec![get_participant(&id)?]
            } else {
                list_participants()?
            };
            if let StatusSource::Chain = source {
                let (network_name, network_def) = resolve_network(&network, &rpc_url, &contract)?;
                let (chain_id, block) = fetch_chain_context(&network_def)?;
                participants.retain(|p| p.last_source == "chain");
                print_json(&serde_json::json!({
                    "source": "chain",
                    "network": network_name,
                    "chain_id": chain_id,
                    "latest_block": block,
                    "participants": participants
                }));
            } else if participants.len() == 1 {
                print_json(&participants[0]);
            } else {
                print_json(&participants);
            }
        }
        Commands::E2e {
            manifest,
            owner,
            stake,
            mode,
            network,
            rpc_url,
            contract,
            register_raw_tx,
            transition_raw_tx,
        } => {
            save_state(&init_state())?;
            let registered = match mode {
                Mode::Local => register_participant(&manifest, &owner, stake)?,
                Mode::Chain => {
                    verify_manifest_signature(&manifest)?;
                    let (network_name, network_def) =
                        resolve_network(&network, &rpc_url, &contract)?;
                    let manifest_payload = manifest::load_manifest(&manifest)?;
                    let action = register_action(
                        &manifest::manifest_field_string(&manifest_payload, "participant_id")?,
                        &manifest::compute_manifest_hash(&manifest)?,
                        &owner,
                        stake,
                    );
                    let artifact = execute_registry_action(
                        &network_name,
                        &network_def,
                        &action,
                        &register_raw_tx,
                    )?;
                    let ctx = WriteContext {
                        source: artifact.source,
                        tx_hash: artifact.tx_hash,
                        block: artifact.block_number,
                        network: Some(artifact.network),
                        chain_id: Some(artifact.chain_id),
                        contract: Some(artifact.contract),
                    };
                    register_participant_with_context(&manifest, &owner, stake, &ctx)?
                }
            };
            let participant_id = registered.participant_id;

            match mode {
                Mode::Local => {
                    transition_participant(
                        &participant_id,
                        "identity_verified",
                        "identity validated",
                    )?;
                }
                Mode::Chain => {
                    let current = get_participant(&participant_id)?;
                    let (network_name, network_def) =
                        resolve_network(&network, &rpc_url, &contract)?;
                    let action = transition_action(
                        &participant_id,
                        &current.state,
                        "identity_verified",
                        "identity validated",
                    );
                    let artifact = execute_registry_action(
                        &network_name,
                        &network_def,
                        &action,
                        &transition_raw_tx,
                    )?;
                    let ctx = WriteContext {
                        source: artifact.source,
                        tx_hash: artifact.tx_hash,
                        block: artifact.block_number,
                        network: Some(artifact.network),
                        chain_id: Some(artifact.chain_id),
                        contract: Some(artifact.contract),
                    };
                    transition_participant_with_context(
                        &participant_id,
                        "identity_verified",
                        "identity validated",
                        &ctx,
                    )?;
                }
            }
            let conformance = run_conformance(&manifest)?;
            if !conformance.passed {
                transition_participant(&participant_id, "restricted", "conformance failed")?;
                print_json(&serde_json::json!({
                    "ok": false,
                    "participant_id": participant_id,
                    "conformance": conformance
                }));
                return Ok(());
            }

            add_attestation(
                &participant_id,
                "conformance_test",
                serde_json::to_value(&conformance).map_err(|e| e.to_string())?,
            )?;
            transition_participant(
                &participant_id,
                "conformance_passed",
                "conformance checks passed",
            )?;
            transition_participant(&participant_id, "probation", "probation routing enabled")?;
            transition_participant(&participant_id, "active", "promotion thresholds met")?;

            let participant = get_participant(&participant_id)?;
            print_json(&serde_json::json!({
                "ok": true,
                "participant": participant,
                "conformance": conformance
            }));
        }
        Commands::Sign { manifest } => {
            let file = sign_manifest(&manifest)?;
            print_json(&serde_json::json!({"ok": true, "signature_file": file}));
        }
        Commands::VerifySignature { manifest } => {
            verify_manifest_signature(&manifest)?;
            print_json(&serde_json::json!({"ok": true, "verified": true}));
        }
        Commands::ValidateE2e { manifest, owner } => {
            save_state(&init_state())?;
            let mut cases = vec![];

            let happy = (|| -> Result<(), String> {
                sign_manifest(&manifest)?;
                register_participant(&manifest, &owner, 100.0)?;
                transition_participant("provider.example.travel", "identity_verified", "test")?;
                let conf = run_conformance(&manifest)?;
                if !conf.passed {
                    return Err("happy path conformance failed".to_string());
                }
                transition_participant("provider.example.travel", "conformance_passed", "test")?;
                transition_participant("provider.example.travel", "probation", "test")?;
                transition_participant("provider.example.travel", "active", "test")?;
                Ok(())
            })();
            cases.push(
                serde_json::json!({"case":"happy_path","passed":happy.is_ok(),"error":happy.err()}),
            );

            let invalid_transition =
                transition_participant("provider.example.travel", "requested", "invalid");
            cases.push(serde_json::json!({"case":"invalid_transition_rejected","passed":invalid_transition.is_err(),"error":invalid_transition.err()}));

            let restricted_ok =
                transition_participant("provider.example.travel", "restricted", "policy breach");
            let revoked_ok =
                transition_participant("provider.example.travel", "revoked", "persistent breach");
            cases.push(serde_json::json!({"case":"restricted_revoked_flow","passed":restricted_ok.is_ok() && revoked_ok.is_ok()}));

            let all_passed = cases.iter().all(|c| {
                c.get("passed")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            });
            print_json(&serde_json::json!({"ok": all_passed, "cases": cases}));
        }
        Commands::CommitmentRecord {
            commitment_id,
            objective_ref,
            offer_id,
            accepted_by,
            provider_id,
            assigned_verifier,
            settlement_authority,
            settlement_state,
            mode,
            network,
            rpc_url,
            contract,
            raw_tx,
        } => {
            let req = RecordCommitment {
                commitment_ref: &commitment_id,
                objective_ref: &objective_ref,
                offer_id: &offer_id,
                accepted_by: &accepted_by,
                provider_id: &provider_id,
                assigned_verifier: &assigned_verifier,
                settlement_authority: &settlement_authority,
                settlement_state: &settlement_state,
            };
            let record = match mode {
                Mode::Local => record_commitment(&req)?,
                Mode::Chain => {
                    let (network_name, network_def) =
                        resolve_network(&network, &rpc_url, &contract)?;
                    let action = register_action(&commitment_id, "commitment", &accepted_by, 0.0);
                    let artifact =
                        execute_registry_action(&network_name, &network_def, &action, &raw_tx)?;
                    let ctx = DisputeWriteContext {
                        source: artifact.source,
                        tx_hash: artifact.tx_hash,
                        block: artifact.block_number,
                        network: Some(artifact.network),
                        chain_id: Some(artifact.chain_id),
                        contract: Some(artifact.contract),
                    };
                    record_commitment_with_context(&req, &ctx)?
                }
            };
            print_json(&record);
        }
        Commands::DisputeOpen {
            commitment_id,
            reason,
            evidence,
            raised_by,
            response_deadline,
            mode,
            network,
            rpc_url,
            contract,
            raw_tx,
        } => {
            let evidence_file = read_evidence_file(&evidence)?;
            let statement = evidence_file.statement.clone();
            let refs = to_evidence_refs(evidence_file.evidence_refs);
            if refs.is_empty() {
                return Err(format!("evidence file {evidence} has no evidence_refs"));
            }
            let record = match mode {
                Mode::Local => open_dispute(OpenDispute {
                    commitment_ref: &commitment_id,
                    raised_by: &raised_by,
                    reason_code: &reason,
                    statement,
                    evidence_refs: refs,
                    response_deadline: &response_deadline,
                })?,
                Mode::Chain => {
                    let (network_name, network_def) =
                        resolve_network(&network, &rpc_url, &contract)?;
                    let dispute_id = derive_dispute_id(&commitment_id, &reason);
                    let root = evidence_root(&refs);
                    let action =
                        open_dispute_action(&dispute_id, &commitment_id, &root, &response_deadline);
                    let artifact =
                        execute_registry_action(&network_name, &network_def, &action, &raw_tx)?;
                    let ctx = DisputeWriteContext {
                        source: artifact.source,
                        tx_hash: artifact.tx_hash,
                        block: artifact.block_number,
                        network: Some(artifact.network),
                        chain_id: Some(artifact.chain_id),
                        contract: Some(artifact.contract),
                    };
                    open_dispute_with_context(
                        OpenDispute {
                            commitment_ref: &commitment_id,
                            raised_by: &raised_by,
                            reason_code: &reason,
                            statement,
                            evidence_refs: refs,
                            response_deadline: &response_deadline,
                        },
                        &ctx,
                    )?
                }
            };
            print_json(&record);
        }
        Commands::DisputeStatus { dispute_id } => {
            let record = get_dispute(&dispute_id)?;
            print_json(&record);
        }
        Commands::DisputeResolve {
            dispute_id,
            decision,
            resolution,
            resolved_by,
            mode,
            network,
            rpc_url,
            contract,
            raw_tx,
        } => {
            let resolution_file = read_evidence_file(&resolution)?;
            let refs = to_evidence_refs(resolution_file.resolution_evidence_refs);
            let record = match mode {
                Mode::Local => resolve_dispute(&dispute_id, &decision, &resolved_by, refs)?,
                Mode::Chain => {
                    let (network_name, network_def) =
                        resolve_network(&network, &rpc_url, &contract)?;
                    let root = evidence_root(&refs);
                    let action = resolve_dispute_action(&dispute_id, &decision, &root);
                    let artifact =
                        execute_registry_action(&network_name, &network_def, &action, &raw_tx)?;
                    let ctx = DisputeWriteContext {
                        source: artifact.source,
                        tx_hash: artifact.tx_hash,
                        block: artifact.block_number,
                        network: Some(artifact.network),
                        chain_id: Some(artifact.chain_id),
                        contract: Some(artifact.contract),
                    };
                    resolve_dispute_with_context(&dispute_id, &decision, &resolved_by, refs, &ctx)?
                }
            };
            print_json(&record);
        }
    }

    Ok(())
}
