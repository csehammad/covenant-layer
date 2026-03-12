use crate::config::NetworkDefinition;
use crate::contracts::RegistryAction;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainArtifact {
    pub network: String,
    pub chain_id: u64,
    pub contract: String,
    pub tx_hash: String,
    pub block_number: u64,
    pub source: String,
}

fn parse_hex_u64(value: &str) -> Result<u64, String> {
    u64::from_str_radix(value.trim_start_matches("0x"), 16)
        .map_err(|e| format!("Failed to parse hex number '{value}': {e}"))
}

fn rpc_call(url: &str, method: &str, params: Value) -> Result<Value, String> {
    let client = Client::new();
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });
    let res = client
        .post(url)
        .json(&payload)
        .send()
        .map_err(|e| format!("RPC request failed: {e}"))?;
    let value: Value = res
        .json()
        .map_err(|e| format!("RPC response parse failed: {e}"))?;
    if let Some(err) = value.get("error") {
        return Err(format!("RPC error from {method}: {err}"));
    }
    value
        .get("result")
        .cloned()
        .ok_or_else(|| format!("Missing result for method {method}"))
}

pub fn fetch_chain_context(network: &NetworkDefinition) -> Result<(u64, u64), String> {
    let chain_id_hex = rpc_call(&network.rpc_url, "eth_chainId", json!([]))?
        .as_str()
        .ok_or_else(|| "eth_chainId result not string".to_string())?
        .to_string();
    let block_hex = rpc_call(&network.rpc_url, "eth_blockNumber", json!([]))?
        .as_str()
        .ok_or_else(|| "eth_blockNumber result not string".to_string())?
        .to_string();
    Ok((parse_hex_u64(&chain_id_hex)?, parse_hex_u64(&block_hex)?))
}

fn wait_for_receipt(
    network: &NetworkDefinition,
    tx_hash: &str,
    max_attempts: u64,
) -> Result<Option<u64>, String> {
    for _ in 0..max_attempts {
        let receipt = rpc_call(
            &network.rpc_url,
            "eth_getTransactionReceipt",
            json!([tx_hash]),
        )?;
        if receipt.is_null() {
            sleep(Duration::from_secs(2));
            continue;
        }
        let block_hex = receipt
            .get("blockNumber")
            .and_then(Value::as_str)
            .ok_or_else(|| "Receipt missing blockNumber".to_string())?;
        return Ok(Some(parse_hex_u64(block_hex)?));
    }
    Ok(None)
}

pub fn execute_registry_action(
    network_name: &str,
    network: &NetworkDefinition,
    action: &RegistryAction,
    raw_tx: &Option<String>,
) -> Result<ChainArtifact, String> {
    let raw = raw_tx.as_ref().ok_or_else(|| {
        "Chain mode requires --raw-tx (no simulated submissions allowed)".to_string()
    })?;
    let (chain_id, latest_block) = fetch_chain_context(network)?;
    let tx_hash = rpc_call(&network.rpc_url, "eth_sendRawTransaction", json!([raw]))?
        .as_str()
        .ok_or_else(|| "eth_sendRawTransaction result not string".to_string())?
        .to_string();
    let block_number = wait_for_receipt(network, &tx_hash, 10)?.unwrap_or(latest_block);
    let _ = action;
    Ok(ChainArtifact {
        network: network_name.to_string(),
        chain_id,
        contract: network.registry_contract.clone(),
        tx_hash,
        block_number,
        source: "chain".to_string(),
    })
}
