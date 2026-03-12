use crate::constants::REQUIRED_STATES;
use serde_yaml::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn load_manifest(path: &str) -> Result<Value, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("Manifest file not found: {path}"));
    }
    let raw = fs::read_to_string(p).map_err(|e| format!("Failed to read manifest: {e}"))?;
    serde_yaml::from_str::<Value>(&raw).map_err(|e| format!("Failed to parse YAML: {e}"))
}

fn has_key(map: &serde_yaml::Mapping, key: &str) -> bool {
    map.contains_key(Value::String(key.to_string()))
}

pub fn validate_manifest(manifest: &Value) -> Vec<String> {
    let mut errors = vec![];
    let Some(map) = manifest.as_mapping() else {
        errors.push("manifest must be a YAML map/object".to_string());
        return errors;
    };

    let required = [
        "manifest_version",
        "conformance_profile",
        "manifest_id",
        "participant_id",
        "participant_type",
        "issued_at",
        "expires_at",
        "identity",
        "keys",
        "capabilities",
        "commitment_policy",
        "evidence_profile",
        "service_interfaces",
        "registry_anchor",
        "admission_status",
        "signature",
    ];
    for field in required {
        if !has_key(map, field) {
            errors.push(format!("missing required field: {field}"));
        }
    }

    if let Some(participant_id) = map.get(Value::String("participant_id".into())) {
        let participant_id = participant_id.as_str().unwrap_or("");
        if !participant_id.contains('.') {
            errors.push(
                "participant_id should be namespaced (e.g. provider.example.travel)".to_string(),
            );
        }
    }

    if let Some(admission) = map.get(Value::String("admission_status".into())) {
        if let Some(admission_map) = admission.as_mapping() {
            if let Some(state) = admission_map.get(Value::String("state".into())) {
                let state = state.as_str().unwrap_or("");
                if !REQUIRED_STATES.contains(&state) {
                    errors.push(format!("invalid admission_status.state: {state}"));
                }
            }
        }
    }

    if let Some(signature) = map.get(Value::String("signature".into())) {
        if let Some(signature_map) = signature.as_mapping() {
            let mode = signature_map
                .get(Value::String("mode".into()))
                .and_then(Value::as_str)
                .unwrap_or("");
            if mode != "detached" {
                errors.push("signature.mode must be 'detached'".to_string());
            }
            let file = signature_map
                .get(Value::String("file".into()))
                .and_then(Value::as_str)
                .unwrap_or("");
            if file.is_empty() {
                errors.push("signature.file is required".to_string());
            }
        }
    }

    errors
}

pub fn manifest_field_string(manifest: &Value, key: &str) -> Result<String, String> {
    manifest
        .as_mapping()
        .and_then(|m| m.get(Value::String(key.to_string())))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("Missing or non-string field: {key}"))
}

pub fn compute_manifest_hash(path: &str) -> Result<String, String> {
    let raw =
        fs::read_to_string(path).map_err(|e| format!("Failed to read manifest for hash: {e}"))?;
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

pub fn sign_manifest(path: &str) -> Result<String, String> {
    let manifest = load_manifest(path)?;
    let signature_file = manifest
        .as_mapping()
        .and_then(|m| m.get(Value::String("signature".into())))
        .and_then(Value::as_mapping)
        .and_then(|s| s.get(Value::String("file".into())))
        .and_then(Value::as_str)
        .ok_or_else(|| "signature.file missing in manifest".to_string())?;
    let hash = compute_manifest_hash(path)?;
    let sig_path = Path::new(path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(signature_file);
    fs::write(&sig_path, format!("{hash}\n"))
        .map_err(|e| format!("Failed writing signature file: {e}"))?;
    Ok(sig_path.to_string_lossy().to_string())
}

pub fn verify_manifest_signature(path: &str) -> Result<(), String> {
    let manifest = load_manifest(path)?;
    let signature_file = manifest
        .as_mapping()
        .and_then(|m| m.get(Value::String("signature".into())))
        .and_then(Value::as_mapping)
        .and_then(|s| s.get(Value::String("file".into())))
        .and_then(Value::as_str)
        .ok_or_else(|| "signature.file missing in manifest".to_string())?;
    let expected_hash = compute_manifest_hash(path)?;
    let sig_path = Path::new(path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(signature_file);
    let sig_raw = fs::read_to_string(&sig_path)
        .map_err(|e| format!("Failed reading signature file {}: {e}", sig_path.display()))?;
    let sig = sig_raw.trim();
    if sig != expected_hash {
        return Err(format!(
            "Detached signature mismatch. expected='{expected_hash}', got='{sig}'"
        ));
    }
    Ok(())
}
