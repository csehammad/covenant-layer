use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDefinition {
    pub chain_id: u64,
    pub rpc_url: String,
    pub registry_contract: String,
    pub explorer_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworksFile {
    pub default_network: String,
    pub networks: HashMap<String, NetworkDefinition>,
}

pub fn load_networks() -> Result<NetworksFile, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("networks.yaml");
    let raw =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read networks.yaml: {e}"))?;
    serde_yaml::from_str::<NetworksFile>(&raw)
        .map_err(|e| format!("Failed to parse networks.yaml: {e}"))
}

pub fn resolve_network(
    name_override: &Option<String>,
    rpc_url_override: &Option<String>,
    contract_override: &Option<String>,
) -> Result<(String, NetworkDefinition), String> {
    let file = load_networks()?;
    let name = name_override
        .clone()
        .unwrap_or_else(|| file.default_network.clone());
    let mut network = file
        .networks
        .get(&name)
        .cloned()
        .ok_or_else(|| format!("Unknown network: {name}"))?;

    if let Some(rpc) = rpc_url_override {
        network.rpc_url = rpc.clone();
    }
    if let Some(contract) = contract_override {
        network.registry_contract = contract.clone();
    }
    Ok((name, network))
}
