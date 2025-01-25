use std::collections::HashMap;

use crate::models::multi_sig_invite::MultiSigInviteStatus;
use crate::models::multi_sig_tx::{
    CkbTransaction, TRANSACTION_STATUS_COMMITED, TRANSACTION_STATUS_FAILED,
    TRANSACTION_STATUS_PENDING, TRANSACTION_STATUS_REJECT,
};
use crate::repositories::address_book::AddressBookDao;
use crate::repositories::ckb::{
    add_signature_to_witness, get_ckb_network, get_live_cell, get_multisig_config,
    get_multisig_script_hash, send_transaction,
};
use crate::repositories::db::DB_POOL;
use crate::serialize::multi_sig_account::{
    InviteInfo, InviteStatusReq, ListSignerRes, MultiSigAccountUpdateReq, TransactionFilters,
    UpdateTransactionStatusReq, UpdateTransactionStatusRes,
};
use crate::serialize::transaction::{ListTransactionsRes, TransactionInfo, TransactionSumary};
use crate::serialize::PaginationRes;
use crate::{
    models::multi_sig_account::MultiSigInfo,
    repositories::multi_sig_account::MultiSigDao,
    serialize::{error::AppError, multi_sig_account::NewMultiSigAccountReq},
};

use ckb_sdk::Address;
use ckb_sdk::AddressPayload;
use ckb_types::bytes::Bytes;
use ckb_types::core::{ScriptHashType, TransactionView};
use ckb_types::packed::Transaction;
use ckb_types::prelude::{IntoTransactionView, Pack, Unpack};

#[derive(Clone, Debug)]
pub struct MultiSigSrv {
    multi_sig_dao: MultiSigDao,
    address_book_dao: AddressBookDao,
}

impl MultiSigSrv {
    pub fn new(multi_sig_dao: MultiSigDao, address_book_dao: AddressBookDao) -> Self {
        MultiSigSrv {
            multi_sig_dao: multi_sig_dao.clone(),
            address_book_dao: address_book_dao.clone(),
        }
    }

    pub async fn request_multi_sig_info_for_signer(
        &self,
        address: &str,
        signer: &str,
    ) -> Result<MultiSigInfo, AppError> {
        match self
            .multi_sig_dao
            .request_multi_sig_info_by_user(address, signer)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?
        {
            Some(info) => Ok(info),
            None => Err(AppError::new(404).message("not found")),
        }
    }

    pub async fn request_multi_sig_info(&self, address: &str) -> Result<MultiSigInfo, AppError> {
        match self
            .multi_sig_dao
            .request_multi_sig_info(&address.to_owned())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?
        {
            Some(info) => Ok(info),
            None => Err(AppError::new(404).message("not found")),
        }
    }

    pub async fn request_list_signers(
        &self,
        address: &str,
        signer: &str,
    ) -> Result<ListSignerRes, AppError> {
        let mut result = ListSignerRes {
            signers: [].to_vec(),
            invites: [].to_vec(),
        };

        match self
            .multi_sig_dao
            .request_list_signers(&address.to_owned(), &signer.to_owned())
            .await
        {
            Ok(signers) => result.signers = signers,
            Err(err) => return Err(AppError::new(500).message(&err.to_string())),
        }

        match self
            .multi_sig_dao
            .get_invites_by_multisig(&address.to_owned())
            .await
        {
            Ok(invites) => result.invites = invites,
            Err(err) => return Err(AppError::new(500).message(&err.to_string())),
        }

        Ok(result)
    }

    pub async fn request_list_accounts(
        &self,
        signer_address: &str,
    ) -> Result<Vec<MultiSigInfo>, AppError> {
        self.multi_sig_dao
            .request_list_accounts(&signer_address.to_owned())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
    }

    pub async fn request_list_transactions(
        &self,
        user_address: &str,
        multisig_address: &str,
        filters: TransactionFilters,
    ) -> Result<ListTransactionsRes, AppError> {
        let limit: i64 = filters.limit.unwrap_or(10);
        let page: i64 = filters.page.unwrap_or(1);

        let res = self
            .multi_sig_dao
            .request_list_transactions(user_address, multisig_address, filters.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
            .unwrap();

        let mut results: Vec<TransactionInfo> = vec![];
        for tx in res {
            let tx_info: ckb_jsonrpc_types::TransactionView =
                serde_json::from_str(tx.payload.as_str()).map_err(|err| {
                    AppError::new(400)
                        .cause(err)
                        .message("invalid transaction json")
                })?;
            let tx_view = Transaction::from(tx_info.clone().inner).into_view();
            let first_output = tx_view.outputs().get(0).unwrap();
            let lock = first_output.lock();
            let address = Address::new(
                get_ckb_network(),
                AddressPayload::new_full(
                    lock.hash_type().try_into().unwrap(),
                    lock.code_hash(),
                    lock.args().unpack(),
                ),
                true,
            );

            let signatures = self
                .multi_sig_dao
                .get_list_signatures_by_txid(&tx.transaction_id)
                .await
                .unwrap();

            let refusers = self
                .multi_sig_dao
                .get_list_rejected_by_txid(&tx.transaction_id)
                .await
                .unwrap();

            let mut errors = None;
            if tx.status.eq(&TRANSACTION_STATUS_FAILED) {
                errors = Some(
                    self.multi_sig_dao
                        .get_errors_by_txid(&tx.transaction_id)
                        .await
                        .unwrap(),
                );
            }

            results.push(TransactionInfo {
                transaction_id: tx.clone().transaction_id,
                multi_sig_address: tx.multi_sig_address,
                to_address: address.to_string(),
                confirmed: signatures
                    .iter()
                    .map(|sig| sig.signer_address.clone())
                    .collect(),
                status: tx.status,
                payload: tx.payload,
                amount: first_output.capacity().unpack(),
                created_at: tx.created_at.timestamp(),
                rejected: refusers
                    .iter()
                    .map(|sig| sig.signer_address.clone())
                    .collect(),
                errors,
            })
        }

        let total_record = self
            .multi_sig_dao
            .get_total_record_by_filters(user_address, multisig_address, filters.clone())
            .await
            .unwrap_or(0);

        let total_page = total_record as f64 / limit as f64;
        Ok(ListTransactionsRes {
            transactions: results,
            pagination: PaginationRes {
                page,
                limit,
                total_records: total_record,
                total_page: total_page.ceil() as i64,
            },
        })
    }

    pub async fn create_new_account(
        &self,
        user_address: &String,
        req: NewMultiSigAccountReq,
    ) -> Result<MultiSigInfo, AppError> {
        let (sender, multi_sig_witness_data) =
            get_multisig_config(req.signers.clone(), req.threshold as u8)?;

        let mut client = DB_POOL.clone().get().await.unwrap();

        if let Some(_signer) = self
            .multi_sig_dao
            .request_multi_sig_info(&sender.to_string())
            .await
            .unwrap()
        {
            return Err(AppError::new(500).message("Account already exists"));
        }

        let transaction = client.transaction().await.unwrap();

        let account_info: MultiSigInfo = match self
            .multi_sig_dao
            .create_new_account(
                &transaction,
                &sender.to_string(),
                &multi_sig_witness_data,
                &req,
            )
            .await
        {
            Ok(a) => a,
            Err(err) => {
                transaction.rollback().await.unwrap();
                return Err(AppError::new(500).message(&err.to_string()));
            }
        };

        for signer in &req.signers {
            match self
                .multi_sig_dao
                .get_signer(&signer.address, &account_info.multi_sig_address)
                .await
            {
                Ok(_signer) => {
                    if let Some(_signer) = _signer {
                        continue;
                    }
                }

                Err(err) => {
                    return Err(AppError::new(500).message(&err.to_string()));
                }
            }

            if signer.address.eq(user_address) {
                match self
                    .multi_sig_dao
                    .add_new_signer(
                        &transaction,
                        &account_info.multi_sig_address,
                        &signer.address,
                    )
                    .await
                {
                    Ok(_) => continue,
                    Err(err) => {
                        transaction.rollback().await.unwrap();
                        return Err(AppError::new(500).message(&err.to_string()));
                    }
                }
            }

            // Check and create a new address book
            if let Ok(address_book) = self
                .address_book_dao
                .get_address(user_address, &signer.address)
                .await
            {
                if address_book.is_none() {
                    let _ = self
                        .address_book_dao
                        .add_address(user_address, &signer.address, &signer.name)
                        .await;
                }
            }

            // Add signer to invite table
            match self
                .multi_sig_dao
                .add_new_invite(
                    &transaction,
                    &account_info.multi_sig_address,
                    &signer.address,
                    MultiSigInviteStatus::PENDING as i16,
                )
                .await
            {
                Ok(_) => continue,
                Err(err) => {
                    transaction.rollback().await.unwrap();
                    return Err(AppError::new(500).message(&err.to_string()));
                }
            }
        }

        transaction.commit().await.unwrap();
        Ok(account_info)
    }

    pub async fn update_account(
        &self,
        user_address: &String,
        req: MultiSigAccountUpdateReq,
    ) -> Result<MultiSigInfo, AppError> {
        let multisig_info = self
            .multi_sig_dao
            .request_multi_sig_info(&req.clone().multi_sig_address)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;

        if multisig_info.is_none() {
            return Err(AppError::new(500).message("Account not found."));
        }

        let mut info = multisig_info.unwrap();
        let signer = self
            .multi_sig_dao
            .get_signer(user_address, &req.clone().multi_sig_address)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;

        if signer.is_none() {
            return Err(
                AppError::new(500).message("You are not the signer of this multisig address.")
            );
        }

        match self
            .multi_sig_dao
            .update_account(req.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?
        {
            true => {
                info.name = req.clone().name;
                Ok(info)
            }
            false => Err(AppError::new(500).message("Update account failed")),
        }
    }

    async fn validate_outpoints(
        &self,
        outpoints: &[ckb_jsonrpc_types::OutPoint],
    ) -> Result<String, AppError> {
        let mut multi_sig_address = "".to_string();
        for outpoint in outpoints.iter().cloned() {
            let cell_with_status = get_live_cell(outpoint, false).await.unwrap();
            if cell_with_status.status.ne(&"live".to_owned()) {
                return Err(AppError::new(400).message("invalid outpoint - consumed"));
            }

            let address = Address::new(
                get_ckb_network(),
                AddressPayload::new_full(
                    ScriptHashType::Type,
                    get_multisig_script_hash().pack(),
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
        Ok(multi_sig_address)
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

        Ok(())
    }

    async fn sync_status_after_broadcast(
        &self,
        txid: &String,
        payload: &String,
    ) -> Result<(), AppError> {
        match self
            .multi_sig_dao
            .sync_status_after_broadcast(txid, payload)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                self.save_transaction_error("syncStatus", &txid.to_owned(), &err.to_string())
                    .await;
                Err(AppError::new(500).message(&err.to_string()))
            }
        }
    }

    async fn broadcast_tx(
        &self,
        json_tx: ckb_jsonrpc_types::TransactionView,
    ) -> Result<(), AppError> {
        let result: Result<ckb_types::H256, ckb_sdk::RpcError> =
            send_transaction(json_tx.inner, None).await;

        if let Err(err) = result {
            let tx_id = json_tx.hash.to_string();
            self.save_transaction_error("sendTransaction", &tx_id, &err.to_string())
                .await;
            return Err(AppError::new(500).cause(err).message("Submit tx failed"));
        }

        Ok(())
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
        let tx: TransactionView = Transaction::from(tx_info.clone().inner).into_view();
        let tx_id = tx_info.hash.to_string();

        let outpoints: Vec<ckb_jsonrpc_types::OutPoint> = tx
            .input_pts_iter()
            .map(ckb_jsonrpc_types::OutPoint::from)
            .collect();

        // validate outpoints status from CKB node
        let multi_sig_address = self.validate_outpoints(&outpoints).await?;

        // Validate if user is one of multi-sig signers
        self.validate_signer(signer_address, &multi_sig_address)
            .await?;
        let multi_sig_info = self.request_multi_sig_info(&multi_sig_address).await?;

        let ckb_tx = self
            .multi_sig_dao
            .create_new_transfer(
                &multi_sig_address.to_string(),
                &tx_id,
                payload,
                signer_address,
                signature,
            )
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;

        // check if threshold is one => broadcast tx immediately
        let _ = self.check_threshold(&multi_sig_info, &tx).await;

        Ok(ckb_tx)
    }

    async fn check_threshold(
        &self,
        multi_sig_info: &MultiSigInfo,
        tx: &TransactionView,
    ) -> Result<(), AppError> {
        let tx_id = hex::encode(tx.hash().raw_data());

        // check if threshold is reached => broadcast tx
        let ckb_signatures = self
            .multi_sig_dao
            .get_list_signatures_by_txid(&tx_id)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;
        if ckb_signatures
            .len()
            .ge(&(multi_sig_info.threshold as usize))
        {
            let signatures = ckb_signatures
                .iter()
                .map(|s| Bytes::from(hex::decode(s.signature.clone()).unwrap()))
                .collect();

            // Add Signatures to witness
            let tx = add_signature_to_witness(
                multi_sig_info.threshold as usize,
                tx,
                &multi_sig_info.multi_sig_witness_data,
                signatures,
            )
            .map_err(|err| {
                AppError::new(500)
                    .cause(err)
                    .message("add signature to witness failed")
            })?;

            let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
            self.broadcast_tx(json_tx.clone()).await?;

            self.sync_status_after_broadcast(
                &tx_id,
                &serde_json::to_string_pretty(&json_tx).unwrap(),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn submit_signature(
        &self,
        signer_address: &String,
        signature: &str,
        txid: &str,
    ) -> Result<CkbTransaction, AppError> {
        let transaction = self
            .multi_sig_dao
            .get_tx_by_hash_and_signer(signer_address, txid)
            .await
            .unwrap();

        if transaction.is_none() {
            return Err(AppError::new(404).message("Transaction not found"));
        }

        let ckb_tx = transaction.unwrap();
        if ckb_tx.status.ne(&TRANSACTION_STATUS_PENDING) {
            return Err(AppError::new(404).message("Transaction not valid"));
        }

        let tx_info: ckb_jsonrpc_types::TransactionView =
            serde_json::from_str(ckb_tx.payload.as_str()).map_err(|err| {
                AppError::new(400)
                    .cause(err)
                    .message("invalid transaction json")
            })?;

        let tx = Transaction::from(tx_info.clone().inner).into_view();
        let tx_id = tx_info.hash.to_string();

        let outpoints: Vec<ckb_jsonrpc_types::OutPoint> = tx
            .input_pts_iter()
            .map(ckb_jsonrpc_types::OutPoint::from)
            .collect();

        // validate outpoints status from CKB node
        // it should be check threshold on cells instead of checking by tx
        // currently all cells is belong to single multi-sig address so we able
        // to use check threshold by txid
        let validate_outputs_result = self.validate_outpoints(&outpoints).await;
        if let Err(err) = validate_outputs_result {
            self.save_transaction_error(signer_address, &tx_id, &err.to_string())
                .await;
            return Err(AppError::new(500).message(&err.to_string()));
        }

        let multi_sig_address = validate_outputs_result.unwrap();

        // Validate if user is one of multi-sig signers
        if let Err(err) = self
            .validate_signer(signer_address, &multi_sig_address)
            .await
        {
            self.save_transaction_error(signer_address, &tx_id, &err.to_string())
                .await;
            return Err(err);
        }

        let multi_sig_info = self.request_multi_sig_info(&multi_sig_address).await?;

        let ckb_tx: CkbTransaction = self
            .multi_sig_dao
            .add_signature(
                &tx_id,
                &multi_sig_address,
                &ckb_tx.payload,
                signer_address,
                &signature.to_owned(),
            )
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;

        // Check threshold sig
        self.check_threshold(&multi_sig_info, &tx).await?;
        Ok(ckb_tx)
    }

    pub async fn reject_transaction(
        &self,
        signer_address: &str,
        txid: &str,
    ) -> Result<bool, AppError> {
        match self
            .multi_sig_dao
            .get_tx_by_hash_and_signer(signer_address, txid)
            .await
            .unwrap()
        {
            Some(transaction) => {
                if let Some(multisig_info) = self
                    .multi_sig_dao
                    .request_multi_sig_info(&transaction.multi_sig_address)
                    .await
                    .map_err(|err| AppError::new(500).message(&err.to_string()))
                    .unwrap()
                {
                    self.multi_sig_dao
                        .reject_transaction(&txid.to_owned(), &signer_address.to_owned())
                        .await
                        .unwrap();

                    let refusers = self
                        .multi_sig_dao
                        .get_list_rejected_by_txid(&txid.to_owned())
                        .await
                        .map_err(|err| AppError::new(500).message(&err.to_string()))
                        .unwrap();

                    let max_valid_signers = multisig_info.signers - (refusers.len() as i16);

                    if max_valid_signers < multisig_info.threshold {
                        let _ = self
                            .multi_sig_dao
                            .update_transaction_status(&txid.to_owned(), TRANSACTION_STATUS_REJECT)
                            .await
                            .map_err(|err| AppError::new(500).message(&err.to_string()));
                    }

                    return Ok(true);
                }
                Err(AppError::new(404).message("Account not found"))
            }
            None => Err(AppError::new(404).message("Transaction not found")),
        }
    }

    pub async fn get_invites_list(&self, address: &String) -> Result<Vec<InviteInfo>, AppError> {
        let accounts = self
            .multi_sig_dao
            .get_invites_list(&address.clone())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()));

        let mut invites: Vec<InviteInfo> = Vec::new();

        for acc in accounts.unwrap() {
            invites.push(InviteInfo {
                address: address.to_string(),
                multisig_address: acc.multi_sig_address,
                account_name: acc.name,
                signers: acc.signers,
                threshold: acc.threshold,
            })
        }

        Ok(invites)
    }

    pub async fn update_invite_status(&self, req: InviteStatusReq) -> Result<bool, AppError> {
        let signer_result = self
            .multi_sig_dao
            .get_signer(&req.address, &req.multisig_address)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()));

        let signer = signer_result.unwrap();
        if signer.clone().is_some() {
            return Err(AppError::new(500).message("Signer has accepted"));
        }

        let invite_res = self
            .multi_sig_dao
            .get_invite(&req.address, &req.multisig_address)
            .await;

        let invite = invite_res.unwrap();
        if invite.clone().is_none() {
            return Err(AppError::new(500).message("Invite not found"));
        }

        let status = invite.unwrap().status;
        if status == MultiSigInviteStatus::ACCEPTED as i16
            || status == MultiSigInviteStatus::REJECTED as i16
        {
            return Err(AppError::new(500).message("Status has been updated"));
        }

        let mut client = DB_POOL.clone().get().await.unwrap();
        let transaction: deadpool_postgres::Transaction = client.transaction().await.unwrap();

        match self
            .multi_sig_dao
            .update_invite_status(
                &transaction,
                req.status,
                &req.address,
                &req.multisig_address,
            )
            .await
        {
            Ok(is_ok) => {
                if is_ok && req.status == MultiSigInviteStatus::ACCEPTED as i16 {
                    match self
                        .multi_sig_dao
                        .add_new_signer(&transaction, &req.multisig_address, &req.address)
                        .await
                    {
                        Ok(_) => (),
                        Err(err) => {
                            transaction.rollback().await.unwrap();
                            return Err(AppError::new(500).message(&err.to_string()));
                        }
                    }
                }

                if is_ok {
                    transaction.commit().await.unwrap();
                    return Ok(true);
                }

                transaction.rollback().await.unwrap();
                Err(AppError::new(500).message("Update invite failed"))
            }
            Err(err) => {
                transaction.rollback().await.unwrap();
                Err(AppError::new(500).message(&err.to_string()))
            }
        }
    }

    pub async fn save_transaction_error(
        &self,
        signer_address: &str,
        transacion_id: &str,
        errors: &str,
    ) {
        let _ = self
            .multi_sig_dao
            .update_transaction_status(&transacion_id.to_string(), TRANSACTION_STATUS_FAILED)
            .await;

        let _ = self
            .multi_sig_dao
            .add_errors(
                &signer_address.to_string(),
                &transacion_id.to_string(),
                &errors.to_string(),
            )
            .await;
    }

    pub async fn rp_transaction_summary(
        &self,
        user_address: &String,
        multisig_address: &String,
    ) -> Result<TransactionSumary, AppError> {
        let transactions = self
            .multi_sig_dao
            .get_pending_tx_by_multisig_and_signer(user_address, multisig_address)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
            .unwrap();

        let mut result = TransactionSumary {
            total_tx_pending: transactions.len() as u32,
            total_amount_pending: 0,
        };

        for tx in transactions {
            let tx_info: ckb_jsonrpc_types::TransactionView =
                serde_json::from_str(tx.payload.as_str()).map_err(|err| {
                    AppError::new(400)
                        .cause(err)
                        .message("invalid transaction json")
                })?;
            let tx_view = Transaction::from(tx_info.clone().inner).into_view();
            let first_output: ckb_types::packed::CellOutput = tx_view.outputs().get(0).unwrap();
            result.total_amount_pending += &first_output.capacity().unpack();
        }

        Ok(result)
    }

    pub async fn update_transaction_commited(
        &self,
        req: &UpdateTransactionStatusReq,
    ) -> Result<UpdateTransactionStatusRes, AppError> {
        let mut results: HashMap<String, bool> = HashMap::new();

        for tx_hash in req.tx_hashes.iter() {
            if let Some(transaction) = self
                .multi_sig_dao
                .get_tx_by_hash(tx_hash)
                .await
                .ok()
                .flatten()
            {
                if transaction.status.eq(&TRANSACTION_STATUS_PENDING) {
                    if let Ok(true) = self
                        .multi_sig_dao
                        .update_transaction_status(tx_hash, TRANSACTION_STATUS_COMMITED)
                        .await
                    {
                        results.insert(tx_hash.clone(), true);
                    }
                }
            }
        }

        Ok(UpdateTransactionStatusRes { results })
    }
}
