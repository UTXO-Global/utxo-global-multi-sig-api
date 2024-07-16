use std::str::FromStr;

use crate::config;
use crate::models::multi_sig_tx::CkbTransaction;
use crate::{
    models::multi_sig_account::{MultiSigInfo, MultiSigSigner},
    repositories::multi_sig_account::MultiSigDao,
    serialize::{error::AppError, multi_sig_account::NewMultiSigAccountReq},
};
use ckb_sdk::{unlock::MultisigConfig, Address, NetworkType};
use ckb_types::packed::Transaction;
use ckb_types::prelude::IntoTransactionView;
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
        let address = address.to_lowercase();
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
        let address = address.to_lowercase();
        self.multi_sig_dao
            .request_list_signers(&address.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
    }

    pub async fn request_list_accounts(
        &self,
        signer_address: &String,
    ) -> Result<Vec<MultiSigInfo>, AppError> {
        let signer_address = signer_address.to_lowercase();
        self.multi_sig_dao
            .request_list_accounts(&signer_address.clone())
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

        let network: String = config::get("network");
        let sender = multisig_config.to_address(
            match network.as_str() {
                "mainnet" => NetworkType::Mainnet,
                _ => NetworkType::Testnet,
            },
            None,
        );

        self.multi_sig_dao
            .create_new_account(&sender.to_string(), &req)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
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

        let multi_sig_address = "".to_string();
        let outpoints: Vec<String> = tx.input_pts_iter().map(|input| input.to_string()).collect();
        // TODO validate outpoints status from CKB node

        // TODO Validate if user is one of multi-sig signers

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
}
