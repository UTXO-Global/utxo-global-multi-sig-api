use std::collections::HashMap;

use ckb_sdk::{
    constants,
    transaction::signer::{
        multisig, sighash::Secp256k1Blake160SighashAllSigner, CKBScriptSigner, SignContexts,
    },
    unlock::{MultisigConfig, UnlockError},
    Address, AddressPayload, CodeHashIndex, NetworkInfo, NetworkType, ScriptId, Since,
    TransactionWithScriptGroups,
};
use ckb_types::{bytes::BytesMut, core::ScriptHashType, prelude::Pack};

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
    fn to_address_payload_override(&self, since_absolute_epoch: Option<u64>) -> AddressPayload {
        let hash160 = self.hash160();
        if let Some(absolute_epoch_number) = since_absolute_epoch {
            let since_value = Since::new_absolute_epoch(absolute_epoch_number).value();
            let mut args = BytesMut::from(hash160.as_bytes());
            args.extend_from_slice(&since_value.to_le_bytes()[..]);
            AddressPayload::new_full(
                ScriptHashType::Type,
                get_multisig_script_hash().pack(),
                args.freeze(),
            )
        } else {
            AddressPayload::new_short(CodeHashIndex::Multisig, hash160)
        }
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
            ScriptId::new_type(constants::MULTISIG_TYPE_HASH.clone()),
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
