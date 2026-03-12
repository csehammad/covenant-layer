use crate::models::{ChainState, RegistryState};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

pub fn state_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("state")
        .join("registry.json")
}

pub fn utc_now() -> String {
    Utc::now().to_rfc3339()
}

pub fn init_state() -> RegistryState {
    RegistryState {
        chain: ChainState {
            name: "local-devnet".to_string(),
            block_height: 0,
        },
        participants: Default::default(),
        events: vec![],
    }
}

pub fn load_state() -> Result<RegistryState, String> {
    let path = state_path();
    if !path.exists() {
        return Ok(init_state());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("Failed to read state: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Failed to parse state JSON: {e}"))
}

pub fn save_state(state: &RegistryState) -> Result<(), String> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create state dir: {e}"))?;
    }
    let body = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize state JSON: {e}"))?;
    fs::write(path, body).map_err(|e| format!("Failed to write state: {e}"))
}

pub fn next_block(state: &mut RegistryState) -> u64 {
    state.chain.block_height += 1;
    state.chain.block_height
}
