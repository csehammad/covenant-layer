use clap::{Parser, Subcommand, ValueEnum};
use covenant_framework::chain::{execute_registry_action, fetch_chain_context};
use covenant_framework::config::resolve_network;
use covenant_framework::conformance::run_conformance;
use covenant_framework::contracts::{attestation_action, register_action, transition_action};
use covenant_framework::manifest::{self, sign_manifest, verify_manifest_signature};
use covenant_framework::registry::{
    add_attestation, add_attestation_with_context, get_participant, list_participants,
    register_participant, register_participant_with_context, transition_participant,
    transition_participant_with_context, WriteContext,
};
use covenant_framework::store::{init_state, save_state};

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
    }

    Ok(())
}
