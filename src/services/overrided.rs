use crate::config;
use ckb_sdk::{
    constants,
    core::TransactionBuilder,
    transaction::{
        handler::{HandlerContext, HandlerContexts, ScriptHandler},
        signer::{
            multisig, sighash::Secp256k1Blake160SighashAllSigner, CKBScriptSigner, SignContexts,
        },
    },
    tx_builder::TxBuilderError,
    unlock::{MultisigConfig, UnlockError},
    Address, AddressPayload, NetworkInfo, NetworkType, ScriptGroup, ScriptId,
    TransactionWithScriptGroups,
};
use ckb_types::{
    bytes::BytesMut,
    core::{DepType, ScriptHashType},
    h256,
    packed::{CellDep, OutPoint, Script},
    prelude::{Entity, Pack},
};
use std::collections::HashMap;

use ckb_types::prelude::Builder;

use crate::repositories::ckb::get_multisig_script_hash;

pub trait OverrideMultisigConfig {
    fn to_address_payload_override(&self, since_absolute_epoch: Option<u64>) -> AddressPayload;
    fn to_address_override(
        &self,
        network: NetworkType,
        since_absolute_epoch: Option<u64>,
    ) -> Address {
        let payload = self.to_address_payload_override(since_absolute_epoch);
        Address::new(network, payload, true)
    }
}

impl OverrideMultisigConfig for MultisigConfig {
    fn to_address_payload_override(&self, _since_absolute_epoch: Option<u64>) -> AddressPayload {
        let hash160 = self.hash160();
        let args = BytesMut::from(hash160.as_bytes());
        AddressPayload::new_full(
            ScriptHashType::Type,
            get_multisig_script_hash().pack(),
            args.freeze(),
        )
    }
}

pub struct TransactionSigner {
    unlockers: HashMap<ScriptId, Box<dyn CKBScriptSigner>>,
}

impl TransactionSigner {
    pub fn new(_network: &NetworkInfo) -> Self {
        let mut unlockers = HashMap::default();

        let sighash_script_id = ScriptId::new_type(constants::SIGHASH_TYPE_HASH.clone());
        unlockers.insert(
            sighash_script_id,
            Box::new(Secp256k1Blake160SighashAllSigner {}) as Box<_>,
        );

        unlockers.insert(
            ScriptId::new_type(get_multisig_script_hash()),
            Box::new(multisig::Secp256k1Blake160MultisigAllSigner {}) as Box<_>,
        );

        Self { unlockers }
    }

    pub fn sign_transaction(
        &self,
        transaction: &mut TransactionWithScriptGroups,
        contexts: &SignContexts,
    ) -> Result<Vec<usize>, UnlockError> {
        let mut signed_groups_indices = vec![];
        if contexts.is_empty() {
            return Ok(signed_groups_indices);
        }
        let mut tx = transaction.get_tx_view().clone();
        for (idx, script_group) in transaction.get_script_groups().iter().enumerate() {
            let script_id = ScriptId::from(&script_group.script);
            if let Some(unlocker) = self.unlockers.get(&script_id) {
                for context in &contexts.contexts {
                    if !unlocker.match_context(context.as_ref()) {
                        continue;
                    }
                    tx = unlocker.sign_transaction(&tx, script_group, context.as_ref())?;
                    signed_groups_indices.push(idx);
                    break;
                }
            }
        }
        transaction.set_tx_view(tx);
        Ok(signed_groups_indices)
    }
}

pub struct OverrideSecp256k1Blake160MultisigAllScriptContext {
    multisig_config: MultisigConfig,
}
impl HandlerContext for OverrideSecp256k1Blake160MultisigAllScriptContext {}
impl OverrideSecp256k1Blake160MultisigAllScriptContext {
    pub fn new(config: MultisigConfig) -> Self {
        Self {
            multisig_config: config,
        }
    }
}

pub struct OverrideSecp256k1Blake160MultisigAllScriptHandler {
    cell_deps: Vec<CellDep>,
}

impl OverrideSecp256k1Blake160MultisigAllScriptHandler {
    pub fn is_match(&self, script: &Script) -> bool {
        script.code_hash() == get_multisig_script_hash().pack()
    }
    pub fn new() -> Result<Self, TxBuilderError> {
        let mut ret = Self { cell_deps: vec![] };
        ret.init(&NetworkInfo::testnet())?; // workaround
        Ok(ret)
    }
}

impl ScriptHandler for OverrideSecp256k1Blake160MultisigAllScriptHandler {
    fn build_transaction(
        &self,
        tx_builder: &mut TransactionBuilder,
        script_group: &mut ScriptGroup,
        context: &dyn HandlerContext,
    ) -> Result<bool, TxBuilderError> {
        if !self.is_match(&script_group.script) {
            return Ok(false);
        }
        if let Some(args) = context
            .as_any()
            .downcast_ref::<OverrideSecp256k1Blake160MultisigAllScriptContext>()
        {
            tx_builder.dedup_cell_deps(self.cell_deps.clone());
            let index = script_group.input_indices.first().unwrap();
            let witness = args.multisig_config.placeholder_witness();
            tx_builder.set_witness(*index, witness.as_bytes().pack());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn init(&mut self, _: &NetworkInfo) -> Result<(), TxBuilderError> {
        let network: String = config::get("network");
        match network.as_str() {
            "mainnet" => {
                let out_point = OutPoint::new_builder()
                    .tx_hash(
                        h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c")
                            .pack(),
                    )
                    .index(1u32.pack())
                    .build();
                let cell_dep = CellDep::new_builder()
                    .out_point(out_point)
                    .dep_type(DepType::DepGroup.into())
                    .build();
                self.cell_deps.push(cell_dep);
            }
            _ => {
                let out_point = OutPoint::new_builder()
                    .tx_hash(
                        h256!("0x8f8c79eb6671709633fe6a46de93c0fedc9c1b8a6527a18d3983879542635c9f")
                            .pack(),
                    )
                    .index(3u32.pack())
                    .build();

                let cell_dep = CellDep::new_builder()
                    .out_point(out_point)
                    .dep_type(DepType::Code.into())
                    .build();
                self.cell_deps.push(cell_dep);

                let out_point = OutPoint::new_builder()
                    .tx_hash(
                        h256!("0xe6774580c98c8b15799c628f539ed5722f3bc2b17206c2280e15f99be3c1ad71")
                            .pack(),
                    )
                    .index(0u32.pack())
                    .build();

                let cell_dep = CellDep::new_builder()
                    .out_point(out_point)
                    .dep_type(DepType::Code.into())
                    .build();
                self.cell_deps.push(cell_dep);
            }
        };

        Ok(())
    }
}

pub trait MultiSigHandlerContext {
    fn new_override_multisig(config: MultisigConfig) -> Self;
}

impl MultiSigHandlerContext for HandlerContexts {
    fn new_override_multisig(config: MultisigConfig) -> Self {
        Self {
            contexts: vec![Box::new(
                OverrideSecp256k1Blake160MultisigAllScriptContext::new(config),
            )],
        }
    }
}
