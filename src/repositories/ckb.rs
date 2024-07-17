use crate::config;
use anyhow::anyhow;
use ckb_sdk::unlock::ScriptSignError;
use ckb_sdk::{rpc::CkbRpcClient, NetworkType};
use ckb_types::bytes::Bytes;
use ckb_types::core::TransactionView;
use ckb_types::packed::WitnessArgs;
use ckb_types::prelude::Builder;
use ckb_types::prelude::{Entity, Pack};

pub fn get_ckb_client() -> CkbRpcClient {
    let rpc_url: String = config::get("ckb_rpc");
    CkbRpcClient::new(rpc_url.as_str())
}

pub fn get_ckb_network() -> NetworkType {
    let network: String = config::get("network");
    match network.as_str() {
        "mainnet" => NetworkType::Mainnet,
        _ => NetworkType::Testnet,
    }
}

pub fn add_signature_to_witness(
    threshold: usize,
    tx: &TransactionView,
    multi_sig_witness_data: &String,
    signatures: Vec<Bytes>,
) -> Result<TransactionView, ScriptSignError> {
    // Hardcode input witness idx = 0 while currently we only support simple transfer
    let witness_idx = 0;
    let mut witnesses: Vec<ckb_types::packed::Bytes> = tx.witnesses().into_iter().collect();
    while witnesses.len() <= witness_idx {
        witnesses.push(Default::default());
    }

    let config_data =
        hex::decode(multi_sig_witness_data).expect("decoding multi_sig_witness_data failed");
    let mut zero_lock = vec![0u8; config_data.len() + 65 * threshold];
    zero_lock[0..config_data.len()].copy_from_slice(&config_data);

    // Assume all inputs belongs to multi-sig address => inputs length = signatures length

    // Put signature into witness
    // Hardcode input witness idx = 0 while currently we only support simple transfer
    let witness_idx = 0;
    let witness_data = witnesses[witness_idx].raw_data();
    let mut current_witness: WitnessArgs = if witness_data.is_empty() {
        WitnessArgs::default()
    } else {
        WitnessArgs::from_slice(witness_data.as_ref())?
    };
    let mut lock_field = current_witness
        .lock()
        .to_opt()
        .map(|data| data.raw_data().as_ref().to_vec())
        .unwrap_or(zero_lock);
    if lock_field.len() != config_data.len() + threshold * 65 {
        return Err(ScriptSignError::Other(anyhow!(
            "invalid witness lock field length: {}, expected: {}",
            lock_field.len(),
            config_data.len() + threshold * 65,
        )));
    }
    for signature in signatures {
        let mut idx = config_data.len();
        while idx < lock_field.len() {
            // Put signature into an empty place.
            if lock_field[idx..idx + 65] == signature {
                break;
            } else if lock_field[idx..idx + 65] == [0u8; 65] {
                lock_field[idx..idx + 65].copy_from_slice(signature.as_ref());
                break;
            }
            idx += 65;
        }
        if idx >= lock_field.len() {
            return Err(ScriptSignError::TooManySignatures);
        }
    }

    current_witness = current_witness
        .as_builder()
        .lock(Some(Bytes::from(lock_field)).pack())
        .build();
    witnesses[witness_idx] = current_witness.as_bytes().pack();
    Ok(tx.as_advanced_builder().set_witnesses(witnesses).build())
}
