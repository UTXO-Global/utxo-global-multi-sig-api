use std::str::FromStr;

use crate::config;
use crate::serialize::error::AppError;
use crate::serialize::multi_sig_account::SignerInfo;
use crate::services::constants::TESTNET_MULTISIG_TYPE_HASH;
use crate::services::overrided::OverrideMultisigConfig;
use anyhow::anyhow;
use ckb_jsonrpc_types::{CellWithStatus, OutputsValidator, Transaction};
use ckb_sdk::constants::MULTISIG_TYPE_HASH;
use ckb_sdk::unlock::{MultisigConfig, ScriptSignError};
use ckb_sdk::{rpc::CkbRpcClient, NetworkType};
use ckb_sdk::{Address, RpcError};
use ckb_types::bytes::Bytes;
use ckb_types::core::TransactionView;
use ckb_types::packed::WitnessArgs;
use ckb_types::prelude::Builder;
use ckb_types::prelude::{Entity, Pack};
use ckb_types::{H160, H256};

pub const CKB_TESTNET_EXPLORER_API: &str = "https://testnet-api.explorer.nervos.org/api";
pub const CKB_MAINNET_EXPLORER_API: &str = "https://mainnet-api.explorer.nervos.org/api";
pub const CKB_TESTNET_RPC: &str = "https://testnet.ckb.dev/rpc";
pub const CKB_MAINNET_RPC: &str = "https://mainnet.ckb.dev/rpc";
pub const JOYID_LOCK_SCRIPT_CODE_HASH: &str =
    "d23761b364210735c19c60561d213fb3beae2fd6172743719eff6920e020baac";

pub fn get_explorer_api_url() -> String {
    let network = get_ckb_network();
    if network == NetworkType::Mainnet {
        return CKB_MAINNET_EXPLORER_API.to_owned();
    }

    CKB_TESTNET_EXPLORER_API.to_owned()
}

pub fn get_rpc() -> String {
    let network = get_ckb_network();
    if network == NetworkType::Mainnet {
        return CKB_MAINNET_RPC.to_owned();
    }

    CKB_TESTNET_RPC.to_owned()
}

pub async fn get_ckb_client() -> CkbRpcClient {
    let rpc_url: String = get_rpc();
    tokio::task::spawn_blocking(move || CkbRpcClient::new(&rpc_url))
        .await
        .expect("Failed to create CkbRpcClient")
}

pub async fn get_live_cell(
    out_point: ckb_jsonrpc_types::OutPoint,
    with_data: bool,
) -> Result<CellWithStatus, ckb_sdk::rpc::RpcError> {
    let rpc_url: String = get_rpc();
    tokio::task::spawn_blocking(move || {
        let client = CkbRpcClient::new(&rpc_url);
        client.get_live_cell(out_point, with_data)
    })
    .await
    .unwrap()
}

pub fn get_ckb_network() -> NetworkType {
    let network: String = config::get("network");
    match network.as_str() {
        "mainnet" => NetworkType::Mainnet,
        _ => NetworkType::Testnet,
    }
}

pub async fn send_transaction(
    tx: Transaction,
    outputs_validator: Option<OutputsValidator>,
) -> Result<H256, RpcError> {
    let rpc_url: String = get_rpc();
    tokio::task::spawn_blocking(move || {
        let client = CkbRpcClient::new(&rpc_url);
        client.send_transaction(tx, outputs_validator)
    })
    .await
    .unwrap()
}

pub fn get_multisig_script_hash() -> ckb_types::H256 {
    let network: String = config::get("network");
    match network.as_str() {
        "mainnet" => MULTISIG_TYPE_HASH,
        _ => TESTNET_MULTISIG_TYPE_HASH,
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
        if idx > lock_field.len() {
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

pub fn get_multisig_config(
    signers: Vec<SignerInfo>,
    threshold: u8,
) -> Result<(Address, String), AppError> {
    let mut sighash_addresses: Vec<H160> = vec![];
    let network = get_ckb_network();
    for signer in signers.iter() {
        let address = Address::from_str(signer.address.as_str()).map_err(|_| {
            AppError::new(400).message(&format!("Address {} invalid", signer.address.as_str()))
        })?;

        let address_hash = hex::encode(address.payload().code_hash(Some(network)).as_slice());
        if address_hash.eq(JOYID_LOCK_SCRIPT_CODE_HASH) {
            return Err(AppError::new(500).message("JoyID addresses are currently not supported as signers in this multisig wallet. Please choose a different address to proceed"));
        }

        // https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0021-ckb-address-format/0021-ckb-address-format.md#short-payload-format
        let sighash_address = address.payload().args();
        sighash_addresses.push(H160::from_slice(sighash_address.as_ref()).unwrap());
    }
    let multisig_config =
        MultisigConfig::new_with(sighash_addresses, 0, threshold).map_err(|e| {
            AppError::new(400)
                .cause(e)
                .message("cannot generate multisig address")
        })?;

    let sender = multisig_config.to_address_override(get_ckb_network(), Some(0));
    let mutli_sig_witness_data = hex::encode(multisig_config.to_witness_data());

    Ok((sender, mutli_sig_witness_data))
}
