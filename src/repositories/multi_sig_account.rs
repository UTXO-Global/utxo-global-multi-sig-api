use std::sync::Arc;

use crate::{
    models::{
        multi_sig_account::{MultiSigInfo, MultiSigSigner},
        multi_sig_invite::MultiSigInvite,
        multi_sig_tx::{CkbSignature, CkbTransaction, TransactionError, TransactionReject},
    },
    serialize::multi_sig_account::{
        MultiSigAccountUpdateReq, NewMultiSigAccountReq, TransactionFilters,
    },
};
use chrono::Utc;
use deadpool_postgres::{Client, Pool, PoolError, Transaction};
use tokio_pg_mapper::FromTokioPostgresRow;

#[derive(Clone, Debug)]
pub struct MultiSigDao {
    db: Arc<Pool>,
}

impl MultiSigDao {
    pub fn new(db: Arc<Pool>) -> Self {
        MultiSigDao { db: db.clone() }
    }

    pub async fn request_multi_sig_info(
        &self,
        address: &String,
    ) -> Result<Option<MultiSigInfo>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM multi_sig_info 
            WHERE multi_sig_address=$1;";
        let stmt = client.prepare(_stmt).await?;

        let row = client.query(&stmt, &[&address]).await?.pop();
        Ok(row.map(|row| MultiSigInfo::from_row_ref(&row).unwrap()))
    }

    pub async fn request_multi_sig_info_by_user(
        &self,
        multisig_address: &str,
        user_address: &str,
    ) -> Result<Option<MultiSigInfo>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT msi.* 
            FROM multi_sig_info msi 
            LEFT JOIN multi_sig_signers mss
            ON msi.multi_sig_address = mss.multi_sig_address
            WHERE msi.multi_sig_address=$1 AND mss.signer_address=$2";
        let stmt = client.prepare(_stmt).await?;

        match client
            .query_opt(&stmt, &[&multisig_address, &user_address])
            .await?
        {
            Some(row) => Ok(Some(MultiSigInfo::from_row_ref(&row).unwrap())),
            None => Ok(None),
        }
    }

    pub async fn request_list_signers(
        &self,
        multisig_address: &String,
        user_address: &String,
    ) -> Result<Vec<MultiSigSigner>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM multi_sig_signers 
            WHERE multi_sig_address=(SELECT multi_sig_address FROM multi_sig_signers WHERE multi_sig_address=$1 AND signer_address=$2 LIMIT 1);
        ";
        let stmt = client.prepare(_stmt).await?;

        let signers = client
            .query(&stmt, &[&multisig_address, user_address])
            .await?
            .iter()
            .map(|row| MultiSigSigner::from_row_ref(row).unwrap())
            .collect::<Vec<MultiSigSigner>>();

        Ok(signers)
    }

    pub async fn get_signer(
        &self,
        address: &String,
        multisig_address: &String,
    ) -> Result<Option<MultiSigSigner>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM multi_sig_signers 
            WHERE multi_sig_address=$1 and signer_address = $2;";
        let stmt = client.prepare(_stmt).await?;

        let row = client
            .query(&stmt, &[&multisig_address, &address])
            .await?
            .pop();

        Ok(row.map(|row| MultiSigSigner::from_row_ref(&row).unwrap()))
    }

    pub async fn add_new_signer(
        &self,
        tx: &Transaction<'_>,
        multi_sig_address: &String,
        address: &String,
    ) -> Result<MultiSigSigner, PoolError> {
        let stmt: &str =
            "INSERT INTO multi_sig_signers (multi_sig_address, signer_address) VALUES ($1, $2);";
        tx.execute(stmt, &[multi_sig_address, address]).await?;
        Ok(MultiSigSigner {
            multi_sig_address: multi_sig_address.clone(),
            signer_address: address.to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
    }

    pub async fn request_list_accounts(
        &self,
        address: &String,
    ) -> Result<Vec<MultiSigInfo>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT msi.* FROM multi_sig_info msi
            LEFT JOIN multi_sig_signers mss 
                ON mss.multi_sig_address = msi.multi_sig_address
            WHERE mss.signer_address=$1;";
        let stmt = client.prepare(_stmt).await?;

        let accounts = client
            .query(&stmt, &[&address])
            .await?
            .iter()
            .map(|row| MultiSigInfo::from_row_ref(row).unwrap())
            .collect::<Vec<MultiSigInfo>>();

        Ok(accounts)
    }

    pub async fn request_list_transactions(
        &self,
        user_address: &str,
        multisig_address: &str,
        filters: TransactionFilters,
    ) -> Result<Vec<CkbTransaction>, PoolError> {
        let limit: i64 = filters.limit.unwrap_or(10);
        let page: i64 = filters.page.unwrap_or(1);
        let offset = (page - 1) * limit;

        let client: Client = self.db.get().await?;

        let mut _stmt = "SELECT tx.* FROM transactions tx
            LEFT JOIN multi_sig_signers mss
                ON mss.multi_sig_address = tx.multi_sig_address
            WHERE mss.signer_address=$1 and tx.multi_sig_address=$2"
            .to_string();

        if let Some(status) = filters.status {
            let statuses: Vec<&str> = status.split(",").collect();
            _stmt = format!("{} AND tx.status IN ({})", _stmt, statuses.join(","));
        }

        if let Some(hash) = filters.tx_hash {
            _stmt = format!("{} AND tx.transaction_id='{}'", _stmt, hash);
        }

        _stmt = format!("{} OFFSET $3 LIMIT $4", _stmt);

        let stmt = client.prepare(&_stmt).await?;

        let txs = client
            .query(&stmt, &[&user_address, &multisig_address, &offset, &limit])
            .await?
            .iter()
            .map(|row| CkbTransaction::from_row_ref(row).unwrap())
            .collect::<Vec<CkbTransaction>>();

        Ok(txs)
    }

    pub async fn get_total_record_by_filters(
        &self,
        user_address: &str,
        multisig_address: &str,
        filters: TransactionFilters,
    ) -> Result<i64, PoolError> {
        let client: Client = self.db.get().await?;

        let mut _stmt = "SELECT COUNT(*) as total_record FROM transactions tx
            LEFT JOIN multi_sig_signers mss
                ON mss.multi_sig_address = tx.multi_sig_address
            WHERE mss.signer_address=$1 and tx.multi_sig_address=$2"
            .to_string();

        if let Some(status) = filters.status {
            let statuses: Vec<&str> = status.split(",").collect();
            _stmt = format!("{} AND tx.status IN ({})", _stmt, statuses.join(","));
        }

        if let Some(hash) = filters.tx_hash {
            _stmt = format!("{} AND tx.transaction_id='{}'", _stmt, hash);
        }

        let stmt = client.prepare(&_stmt).await?;

        let row = client
            .query_one(&stmt, &[&user_address, &multisig_address])
            .await
            .unwrap();

        Ok(row.get(0))
    }

    pub async fn create_new_account(
        &self,
        tx: &Transaction<'_>,
        multi_sig_address: &String,
        mutli_sig_witness_data: &String,
        req: &NewMultiSigAccountReq,
    ) -> Result<MultiSigInfo, PoolError> {
        let stmt: &str = "INSERT INTO multi_sig_info (multi_sig_address, threshold, signers, name, mutli_sig_witness_data) VALUES ($1, $2, $3, $4, $5);";

        tx.execute(
            stmt,
            &[
                multi_sig_address,
                &req.threshold,
                &(req.signers.len() as i16),
                &req.name,
                mutli_sig_witness_data,
            ],
        )
        .await?;
        Ok(MultiSigInfo {
            multi_sig_address: multi_sig_address.clone(),
            threshold: req.threshold,
            signers: req.signers.len() as i16,
            name: req.name.clone(),
            mutli_sig_witness_data: mutli_sig_witness_data.clone(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
    }

    pub async fn update_account(&self, req: MultiSigAccountUpdateReq) -> Result<bool, PoolError> {
        let client: Client = self.db.get().await?;
        let stmt = "UPDATE multi_sig_info SET name = $1 WHERE multi_sig_address = $2";
        let res = client
            .execute(stmt, &[&req.name, &req.multi_sig_address])
            .await?;
        Ok(res > 0)
    }

    pub async fn create_new_transfer(
        &self,
        multi_sig_address: &String,
        transaction_id: &String,
        payload: &String,
        signer_address: &String,
        signature: &String,
    ) -> Result<CkbTransaction, PoolError> {
        let mut client: Client = self.db.get().await?;

        let db_transaction = client.transaction().await?;

        // Create tx
        let _stmt =
            "INSERT INTO transactions (transaction_id, multi_sig_address, payload, status) VALUES ($1, $2, $3, 0);";
        let stmt = db_transaction.prepare(_stmt).await?;
        db_transaction
            .execute(&stmt, &[transaction_id, multi_sig_address, payload])
            .await?;

        // Add first signatures - requester of this new transaction
        let _stmt =
            "INSERT INTO signatures (signer_address, transaction_id, signature) VALUES ($1, $2, $3);";
        let stmt = db_transaction.prepare(_stmt).await?;
        db_transaction
            .execute(&stmt, &[signer_address, transaction_id, &signature])
            .await?;

        db_transaction.commit().await?;

        Ok(CkbTransaction {
            transaction_id: transaction_id.clone(),
            multi_sig_address: multi_sig_address.clone(),
            payload: payload.clone(),
            status: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
    }

    pub async fn get_tx_by_hash(&self, txid: &String) -> Result<Option<CkbTransaction>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM transactions 
            WHERE transaction_id=$1;";
        let stmt = client.prepare(_stmt).await?;

        let row = client.query(&stmt, &[&txid]).await?.pop();
        Ok(row.map(|row| CkbTransaction::from_row_ref(&row).unwrap()))
    }

    pub async fn get_tx_by_hash_and_signer(
        &self,
        user_address: &str,
        txid: &str,
    ) -> Result<Option<CkbTransaction>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * 
            FROM transactions 
            WHERE transaction_id=$1 AND
                multi_sig_address IN (
                    SELECT multi_sig_address FROM multi_sig_signers WHERE signer_address=$2
                )";
        let stmt = client.prepare(_stmt).await?;

        let row = client.query(&stmt, &[&txid, &user_address]).await?.pop();
        Ok(row.map(|row| CkbTransaction::from_row_ref(&row).unwrap()))
    }

    pub async fn add_signature(
        &self,
        transaction_id: &String,
        multi_sig_address: &str,
        payload: &str,
        signer_address: &String,
        signature: &String,
    ) -> Result<CkbTransaction, PoolError> {
        let client: Client = self.db.get().await?;

        // Add signature
        let _stmt =
            "INSERT INTO signatures (signer_address, transaction_id, signature) VALUES ($1, $2, $3);";
        let stmt = client.prepare(_stmt).await?;
        client
            .execute(&stmt, &[signer_address, transaction_id, &signature])
            .await?;

        Ok(CkbTransaction {
            transaction_id: transaction_id.clone(),
            multi_sig_address: multi_sig_address.to_owned(),
            payload: payload.to_owned(),
            status: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
    }

    pub async fn get_matched_signer(
        &self,
        signer_address: &String,
        multi_sig_address: &String,
    ) -> Result<Option<MultiSigSigner>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM multi_sig_signers
            WHERE multi_sig_address=$1 AND signer_address=$2;";
        let stmt = client.prepare(_stmt).await?;

        let row = client
            .query(&stmt, &[&multi_sig_address, &signer_address])
            .await?
            .pop();

        Ok(row.map(|row| MultiSigSigner::from_row_ref(&row).unwrap()))
    }

    pub async fn get_list_signatures_by_txid(
        &self,
        txid: &String,
    ) -> Result<Vec<CkbSignature>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM signatures 
            WHERE transaction_id=$1;";
        let stmt = client.prepare(_stmt).await?;

        let signatures = client
            .query(&stmt, &[&txid])
            .await?
            .iter()
            .map(|row| CkbSignature::from_row_ref(row).unwrap())
            .collect::<Vec<CkbSignature>>();

        Ok(signatures)
    }

    pub async fn sync_status_after_broadcast(
        &self,
        transaction_id: &String,
        payload: &String,
    ) -> Result<CkbTransaction, PoolError> {
        let mut client: Client = self.db.get().await?;
        let db_transaction = client.transaction().await?;

        // Update tx
        let _stmt = "UPDATE transactions SET status = 1, payload = $2  WHERE transaction_id = $1 RETURNING *;";
        let stmt = db_transaction.prepare(_stmt).await?;
        let tx_updated = db_transaction
            .query_one(&stmt, &[transaction_id, payload])
            .await
            .map(|row| CkbTransaction::from_row(row).unwrap())?;

        db_transaction.commit().await?;
        Ok(tx_updated)
    }

    // Invite

    pub async fn get_invites_list(&self, address: &String) -> Result<Vec<MultiSigInfo>, PoolError> {
        let client: Client = self.db.get().await?;
        let _stmt = "
            SELECT msi.* 
            FROM multi_sig_info msi
            LEFT JOIN multi_sig_invites mss 
            ON mss.multi_sig_address = msi.multi_sig_address
            WHERE mss.signer_address=$1 and mss.status = 0
        ";
        let stmt = client.prepare(_stmt).await?;

        let invites = client
            .query(&stmt, &[&address])
            .await?
            .iter()
            .map(|row| MultiSigInfo::from_row_ref(row).unwrap())
            .collect::<Vec<MultiSigInfo>>();

        Ok(invites)
    }

    pub async fn get_invites_by_multisig(
        &self,
        address: &String,
    ) -> Result<Vec<MultiSigInvite>, PoolError> {
        let client: Client = self.db.get().await?;
        let _stmt = "
            SELECT *
            FROM multi_sig_invites
            WHERE multi_sig_address=$1
        ";
        let stmt = client.prepare(_stmt).await?;

        let invites: Vec<MultiSigInvite> = client
            .query(&stmt, &[&address])
            .await?
            .iter()
            .map(|row| MultiSigInvite::from_row_ref(row).unwrap())
            .collect::<Vec<MultiSigInvite>>();

        Ok(invites)
    }

    pub async fn get_invite(
        &self,
        address: &String,
        multisig_address: &String,
    ) -> Result<Option<MultiSigInvite>, PoolError> {
        let client: Client = self.db.get().await?;

        let stmt =
            "SELECT * FROM multi_sig_invites WHERE multi_sig_address=$1 AND signer_address=$2;";
        match client.query_opt(stmt, &[multisig_address, address]).await? {
            Some(row) => {
                let invite = MultiSigInvite::from_row_ref(&row).unwrap();
                Ok(Some(invite))
            }
            None => Ok(None),
        }
    }

    pub async fn update_invite_status(
        &self,
        tx: &Transaction<'_>,
        status: i16,
        address: &String,
        multisig_address: &String,
    ) -> Result<bool, PoolError> {
        let stmt = "UPDATE multi_sig_invites SET status = $1 WHERE multi_sig_address = $2  and signer_address = $3";
        let res = tx
            .execute(stmt, &[&status, multisig_address, address])
            .await?;
        Ok(res > 0)
    }

    pub async fn add_new_invite(
        &self,
        tx: &Transaction<'_>,
        multi_sig_address: &String,
        address: &String,
        status: i16,
    ) -> Result<MultiSigInvite, PoolError> {
        let stmt: &str =
            "INSERT INTO multi_sig_invites (multi_sig_address, signer_address, status) VALUES ($1, $2, $3);";
        tx.execute(stmt, &[multi_sig_address, address, &status])
            .await?;

        Ok(MultiSigInvite {
            multi_sig_address: multi_sig_address.clone(),
            signer_address: address.to_string(),
            status,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
    }

    pub async fn get_pending_tx_by_multisig_and_signer(
        &self,
        user_address: &String,
        multisig_address: &String,
    ) -> Result<Vec<CkbTransaction>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "
            SELECT tx.* FROM transactions tx
            LEFT JOIN signatures sig
            ON tx.transaction_id = sig.transaction_id
            WHERE tx.status=0 AND 
                    tx.multi_sig_address=(
                        SELECT multi_sig_address 
                        FROM multi_sig_signers 
                        WHERE multi_sig_address=$1 AND signer_address=$2 LIMIT 1
                    ) AND
                    sig.signer_address<>$2
        ";
        let stmt = client.prepare(_stmt).await?;

        let transactions: Vec<CkbTransaction> = client
            .query(&stmt, &[&multisig_address, &user_address])
            .await?
            .iter()
            .map(|row| CkbTransaction::from_row_ref(row).unwrap())
            .collect::<Vec<CkbTransaction>>();

        Ok(transactions)
    }

    pub async fn update_transaction_status(
        &self,
        transaction_id: &String,
        status: i16,
    ) -> Result<bool, PoolError> {
        let client: Client = self.db.get().await?;
        let stmt = "UPDATE transactions SET status=$1 WHERE transaction_id=$2";
        Ok(client
            .execute(stmt, &[&status, transaction_id])
            .await
            .unwrap()
            > 0)
    }

    // Transaction errors

    pub async fn add_errors(
        &self,
        signer_address: &String,
        transaction_id: &String,
        errors: &String,
    ) -> Result<bool, PoolError> {
        let client: Client = self.db.get().await?;

        // Create tx
        let stmt =
            "INSERT INTO transaction_errors (transaction_id, signer_address, errors) VALUES ($1, $2, $3);";
        let stmt = client.prepare(stmt).await?;
        Ok(client
            .execute(&stmt, &[transaction_id, signer_address, errors])
            .await
            .unwrap()
            > 0)
    }

    pub async fn get_errors_by_txid(
        &self,
        txid: &String,
    ) -> Result<Vec<TransactionError>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt =
            "SELECT * FROM transaction_errors WHERE transaction_id=$1 ORDER BY created_at DESC";
        let stmt = client.prepare(_stmt).await?;

        let errors = client
            .query(&stmt, &[&txid])
            .await?
            .iter()
            .map(|row| TransactionError::from_row_ref(row).unwrap())
            .collect::<Vec<TransactionError>>();

        Ok(errors)
    }

    pub async fn reject_transaction(
        &self,
        transaction_id: &String,
        signer_address: &String,
    ) -> Result<bool, PoolError> {
        let client: Client = self.db.get().await?;

        // Create tx
        let stmt =
            "INSERT INTO transaction_rejects (transaction_id, signer_address) VALUES ($1, $2);";
        let stmt = client.prepare(stmt).await?;
        Ok(client
            .execute(&stmt, &[transaction_id, signer_address])
            .await
            .unwrap()
            > 0)
    }

    pub async fn get_list_rejected_by_txid(
        &self,
        txid: &String,
    ) -> Result<Vec<TransactionReject>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM transaction_rejects 
            WHERE transaction_id=$1;";
        let stmt = client.prepare(_stmt).await?;

        let refusers = client
            .query(&stmt, &[&txid])
            .await?
            .iter()
            .map(|row| TransactionReject::from_row_ref(row).unwrap())
            .collect::<Vec<TransactionReject>>();

        Ok(refusers)
    }
}
