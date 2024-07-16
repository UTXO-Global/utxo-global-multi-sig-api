use std::str::FromStr;

use crate::models::multi_sig_tx::CkbTransaction;
use crate::repositories::ckb::{get_ckb_client, get_ckb_network};
use crate::{
    models::multi_sig_account::{MultiSigInfo, MultiSigSigner},
    repositories::multi_sig_account::MultiSigDao,
    serialize::{error::AppError, multi_sig_account::NewMultiSigAccountReq},
};
use ckb_sdk::constants::MULTISIG_TYPE_HASH;
use ckb_sdk::AddressPayload;
use ckb_sdk::{unlock::MultisigConfig, Address};
use ckb_types::bytes::Bytes;
use ckb_types::core::ScriptHashType;
use ckb_types::packed::Transaction;
use ckb_types::prelude::{IntoTransactionView, Pack};
use ckb_types::H160;

#[derive(Clone, Debug)]
pub struct MultiSigSrv {
    multi_sig_dao: MultiSigDao,
}

impl MultiSigSrv {
    pub fn new(multi_sig_dao: MultiSigDao) -> Self {
        MultiSigSrv {
            multi_sig_dao: multi_sig_dao.clone(),
        }
    }

    pub async fn request_multi_sig_info(&self, address: &String) -> Result<MultiSigInfo, AppError> {
        match self
            .multi_sig_dao
            .request_multi_sig_info(&address.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?
        {
            Some(info) => Ok(info),
            None => Err(AppError::new(404).message("not found")),
        }
    }

    pub async fn request_list_signers(
        &self,
        address: &String,
    ) -> Result<Vec<MultiSigSigner>, AppError> {
        self.multi_sig_dao
            .request_list_signers(&address.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
    }

    pub async fn request_list_accounts(
        &self,
        signer_address: &String,
    ) -> Result<Vec<MultiSigInfo>, AppError> {
        self.multi_sig_dao
            .request_list_accounts(&signer_address.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
    }

    pub async fn request_list_transactions(
        &self,
        signer_address: &String,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<CkbTransaction>, AppError> {
        self.multi_sig_dao
            .request_list_transactions(&signer_address.clone(), offset, limit)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
    }

    pub async fn create_new_account(
        &self,
        req: NewMultiSigAccountReq,
    ) -> Result<MultiSigInfo, AppError> {
        let mut sighash_addresses: Vec<H160> = vec![];
        for signer in req.signers.iter() {
            let address = Address::from_str(signer.as_str())
                .map_err(|_| AppError::new(400).message("invalid address"))?;
            // https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0021-ckb-address-format/0021-ckb-address-format.md#short-payload-format
            let sighash_address = address.payload().args();
            sighash_addresses.push(H160::from_slice(sighash_address.as_ref()).unwrap());
        }
        let multisig_config = MultisigConfig::new_with(sighash_addresses, 0, req.threshold as u8)
            .map_err(|e| {
            AppError::new(400)
                .cause(e)
                .message("cannot generate multisig address")
        })?;

        let sender = multisig_config.to_address(get_ckb_network(), None);

        self.multi_sig_dao
            .create_new_account(&sender.to_string(), &req)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
    }

    fn validate_outpoints(
        &self,
        outpoints: &Vec<ckb_jsonrpc_types::OutPoint>,
    ) -> Result<String, AppError> {
        let mut multi_sig_address = "".to_string();
        for outpoint in outpoints.clone() {
            let cell_with_status = get_ckb_client().get_live_cell(outpoint, false).unwrap();
            if cell_with_status.status.ne(&"live".to_owned()) {
                return Err(AppError::new(400).message("invalid outpoint - consumed"));
            }

            let address = Address::new(
                get_ckb_network(),
                AddressPayload::new_full(
                    ScriptHashType::Type,
                    MULTISIG_TYPE_HASH.pack(),
                    Bytes::copy_from_slice(
                        cell_with_status.cell.unwrap().output.lock.args.as_bytes(),
                    ),
                ),
                true,
            )
            .to_string();

            if multi_sig_address.is_empty() {
                multi_sig_address = address;
            } else if multi_sig_address.ne(&address) {
                return Err(AppError::new(400).message("invalid outpoint - not owned"));
            }
        }
        return Ok(multi_sig_address);
    }

    async fn validate_signer(
        &self,
        signer_address: &String,
        multi_sig_address: &String,
    ) -> Result<(), AppError> {
        let signer = self
            .multi_sig_dao
            .get_matched_signer(signer_address, multi_sig_address)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;
        if signer.is_none() {
            return Err(AppError::new(401).message("invalid signer"));
        }

        return Ok(());
    }

    pub async fn create_new_transfer(
        &self,
        signer_address: &String,
        signature: &String,
        payload: &String,
    ) -> Result<CkbTransaction, AppError> {
        let tx_info: ckb_jsonrpc_types::TransactionView = serde_json::from_str(payload.as_str())
            .map_err(|err| {
                AppError::new(400)
                    .cause(err)
                    .message("invalid transaction json")
            })?;
        let tx = Transaction::from(tx_info.inner).into_view();
        let tx_id = tx.hash().to_string();

        let outpoints: Vec<ckb_jsonrpc_types::OutPoint> = tx
            .input_pts_iter()
            .map(|outpoint| ckb_jsonrpc_types::OutPoint::from(outpoint))
            .collect();

        // validate outpoints status from CKB node
        let multi_sig_address = self.validate_outpoints(&outpoints)?;

        // Validate if user is one of multi-sig signers
        self.validate_signer(&signer_address, &multi_sig_address)
            .await?;

        let outpoints: Vec<String> = outpoints
            .into_iter()
            .map(|outpoint| format!("{}:{}", outpoint.tx_hash, outpoint.index.value()))
            .collect();
        let ckb_tx = self
            .multi_sig_dao
            .create_new_transfer(
                &multi_sig_address.to_string(),
                outpoints,
                &tx_id,
                payload,
                signer_address,
                signature,
            )
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;

        // TODO: check if threshold is one => broadcast tx immediately

        Ok(ckb_tx)
    }

    async fn get_tx_by_hash(&self, txid: &String) -> Result<CkbTransaction, AppError> {
        match self
            .multi_sig_dao
            .get_tx_by_hash(&txid.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?
        {
            Some(info) => Ok(info),
            None => Err(AppError::new(404).message("tx not found")),
        }
    }

    pub async fn submit_signature(
        &self,
        signer_address: &String,
        signature: &String,
        txid: &String,
    ) -> Result<CkbTransaction, AppError> {
        let ckb_tx = self.get_tx_by_hash(txid).await?;

        // TODO check tx status

        let tx_info: ckb_jsonrpc_types::TransactionView =
            serde_json::from_str(ckb_tx.payload.as_str()).map_err(|err| {
                AppError::new(400)
                    .cause(err)
                    .message("invalid transaction json")
            })?;
        let tx = Transaction::from(tx_info.inner).into_view();
        let tx_id = tx.hash().to_string();

        let outpoints: Vec<ckb_jsonrpc_types::OutPoint> = tx
            .input_pts_iter()
            .map(|outpoint| ckb_jsonrpc_types::OutPoint::from(outpoint))
            .collect();
        // validate outpoints status from CKB node
        let multi_sig_address = self.validate_outpoints(&outpoints)?;

        // Validate if user is one of multi-sig signers
        self.validate_signer(&signer_address, &multi_sig_address)
            .await?;

        let ckb_tx = self
            .multi_sig_dao
            .add_signature(&tx_id, &ckb_tx.payload, signer_address, signature)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;

        // TODO: check if threshold is reached => broadcast tx

        Ok(ckb_tx)
    }
}
