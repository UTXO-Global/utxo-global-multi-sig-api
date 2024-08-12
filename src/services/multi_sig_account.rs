use crate::models::multi_sig_invite::MultiSigInviteStatus;
use crate::models::multi_sig_tx::CkbTransaction;
use crate::repositories::address_book::AddressBookDao;
use crate::repositories::ckb::{
    add_signature_to_witness, get_ckb_network, get_live_cell, get_multisig_config, send_transaction,
};
use crate::repositories::db::DB_POOL;
use crate::serialize::multi_sig_account::{
    InviteInfo, InviteStatusReq, ListSignerRes, MultiSigAccountUpdateReq,
};
use crate::serialize::transaction::TransactionInfo;
use crate::{
    models::multi_sig_account::MultiSigInfo,
    repositories::multi_sig_account::MultiSigDao,
    serialize::{error::AppError, multi_sig_account::NewMultiSigAccountReq},
};
use ckb_sdk::constants::MULTISIG_TYPE_HASH;
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

    pub async fn request_list_signers(&self, address: &str) -> Result<ListSignerRes, AppError> {
        let mut result = ListSignerRes {
            signers: [].to_vec(),
            invites: [].to_vec(),
        };

        match self
            .multi_sig_dao
            .request_list_signers(&address.to_owned())
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
        signer_address: &str,
        multi_sig_address: &str,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<TransactionInfo>, AppError> {
        let res = self
            .multi_sig_dao
            .request_list_transactions(
                &signer_address.to_owned(),
                &multi_sig_address.to_owned(),
                offset,
                limit,
            )
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

            results.push(TransactionInfo {
                transaction_id: tx.transaction_id,
                multi_sig_address: tx.multi_sig_address,
                to_address: address.to_string(),
                confirmed: signatures
                    .iter()
                    .map(|sig| sig.signer_address.clone())
                    .collect(),
                status: tx.status,
                payload: tx.payload,
                amount: first_output.capacity().unpack(),
                created_at: tx.created_at.to_string(),
            })
        }
        Ok(results)
    }

    pub async fn create_new_account(
        &self,
        user_address: &String,
        req: NewMultiSigAccountReq,
    ) -> Result<MultiSigInfo, AppError> {
        let (sender, mutli_sig_witness_data) =
            get_multisig_config(req.signers.clone(), req.threshold as u8)?;

        let mut client = DB_POOL.clone().get().await.unwrap();
        let transaction = client.transaction().await.unwrap();
        let account_info: MultiSigInfo = match self
            .multi_sig_dao
            .create_new_account(
                &transaction,
                &sender.to_string(),
                &mutli_sig_witness_data,
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
        self.multi_sig_dao
            .sync_status_after_broadcast(txid, payload)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?;
        Ok(())
    }

    async fn broadcast_tx(
        &self,
        json_tx: ckb_jsonrpc_types::TransactionView,
    ) -> Result<(), AppError> {
        let result: Result<ckb_types::H256, ckb_sdk::RpcError> =
            send_transaction(json_tx.inner, None).await;

        if let Err(err) = result {
            log::error!(target: "multi_sig_service", "Submit tx failed: {err:?}");
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
        let tx = Transaction::from(tx_info.clone().inner).into_view();
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
        self.check_threshold(&multi_sig_info, &tx).await?;

        Ok(ckb_tx)
    }

    async fn get_tx_by_hash(&self, txid: &str) -> Result<CkbTransaction, AppError> {
        match self
            .multi_sig_dao
            .get_tx_by_hash(&txid.to_owned())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?
        {
            Some(info) => Ok(info),
            None => Err(AppError::new(404).message("tx not found")),
        }
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
            .eq(&(multi_sig_info.threshold as usize))
        {
            let signatures = ckb_signatures
                .iter()
                .map(|s| Bytes::from(hex::decode(s.signature.clone()).unwrap()))
                .collect();

            println!("{:?}", signatures);

            // Add Signatures to witness
            let tx = add_signature_to_witness(
                multi_sig_info.threshold as usize,
                tx,
                &multi_sig_info.mutli_sig_witness_data,
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
        let ckb_tx = self.get_tx_by_hash(txid).await?;

        // TODO check tx status
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
        let multi_sig_address = self.validate_outpoints(&outpoints).await?;

        // Validate if user is one of multi-sig signers
        self.validate_signer(signer_address, &multi_sig_address)
            .await?;
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
}
