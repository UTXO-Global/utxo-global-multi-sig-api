use std::sync::Arc;

use crate::{
    models::{
        multi_sig_account::{MultiSigInfo, MultiSigSigner},
        multi_sig_tx::{CkbSignature, CkbTransaction},
    },
    serialize::multi_sig_account::NewMultiSigAccountReq,
};
use chrono::Utc;
use deadpool_postgres::{Client, Pool, PoolError};
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
        let stmt = client.prepare(&_stmt).await?;

        let row = client.query(&stmt, &[&address]).await?.pop();

        Ok(match row {
            Some(row) => Some(MultiSigInfo::from_row_ref(&row).unwrap()),
            None => None,
        })
    }

    pub async fn request_list_signers(
        &self,
        address: &String,
    ) -> Result<Vec<MultiSigSigner>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM multi_sig_signers 
            WHERE multi_sig_address=$1;";
        let stmt = client.prepare(&_stmt).await?;

        let signers = client
            .query(&stmt, &[&address])
            .await?
            .iter()
            .map(|row| MultiSigSigner::from_row_ref(&row).unwrap())
            .collect::<Vec<MultiSigSigner>>();

        Ok(signers)
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
        let stmt = client.prepare(&_stmt).await?;

        let accounts = client
            .query(&stmt, &[&address])
            .await?
            .iter()
            .map(|row| MultiSigInfo::from_row_ref(&row).unwrap())
            .collect::<Vec<MultiSigInfo>>();

        Ok(accounts)
    }

    pub async fn request_list_transactions(
        &self,
        signer_address: &String,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<CkbTransaction>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT tx.* FROM transactions tx
            LEFT JOIN cells
                ON cells.transaction_id = tx.transaction_id
            LEFT JOIN multi_sig_signers mss
                ON mss.multi_sig_address = cells.multi_sig_address
            WHERE mss.signer_address=$1
            OFFSET $2 LIMIT $3;";
        let stmt = client.prepare(&_stmt).await?;

        let txs = client
            .query(&stmt, &[&signer_address, &offset, &limit])
            .await?
            .iter()
            .map(|row| CkbTransaction::from_row_ref(&row).unwrap())
            .collect::<Vec<CkbTransaction>>();

        Ok(txs)
    }

    pub async fn create_new_account(
        &self,
        multi_sig_address: &String,
        mutli_sig_witness_data: &String,
        req: &NewMultiSigAccountReq,
    ) -> Result<MultiSigInfo, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "INSERT INTO multi_sig_info (multi_sig_address, threshold, signers, name, mutli_sig_witness_data) VALUES ($1, $2, $3, $4, $5);";
        let stmt = client.prepare(&_stmt).await?;

        client
            .execute(
                &stmt,
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

    pub async fn create_new_transfer(
        &self,
        multi_sig_address: &String,
        outpoints: Vec<String>,
        transaction_id: &String,
        payload: &String,
        signer_address: &String,
        signature: &String,
    ) -> Result<CkbTransaction, PoolError> {
        let mut client: Client = self.db.get().await?;

        let db_transaction = client.transaction().await?;

        // Create tx
        let _stmt =
            "INSERT INTO transactions (transaction_id, payload, status) VALUES ($1, $2, 0);";
        let stmt = db_transaction.prepare(&_stmt).await?;
        db_transaction
            .execute(&stmt, &[transaction_id, payload])
            .await?;

        // Create cells from tx info
        for outpoint in outpoints {
            let _stmt =
            "INSERT INTO cells (multi_sig_address, outpoint, transaction_id, status) VALUES ($1, $2, 0);";
            let stmt = db_transaction.prepare(&_stmt).await?;
            db_transaction
                .execute(&stmt, &[multi_sig_address, &outpoint, transaction_id])
                .await?;
        }

        // Add first signatures - requester of this new transaction
        let _stmt =
            "INSERT INTO signatures (signer_address, transaction_id, signature) VALUES ($1, $2, $3);";
        let stmt = db_transaction.prepare(&_stmt).await?;
        db_transaction
            .execute(&stmt, &[signer_address, transaction_id, &signature])
            .await?;

        db_transaction.commit().await?;

        Ok(CkbTransaction {
            transaction_id: transaction_id.clone(),
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
        let stmt = client.prepare(&_stmt).await?;

        let row = client.query(&stmt, &[&txid]).await?.pop();

        Ok(match row {
            Some(row) => Some(CkbTransaction::from_row_ref(&row).unwrap()),
            None => None,
        })
    }

    pub async fn add_signature(
        &self,
        transaction_id: &String,
        payload: &String,
        signer_address: &String,
        signature: &String,
    ) -> Result<CkbTransaction, PoolError> {
        let client: Client = self.db.get().await?;

        // Add signature
        let _stmt =
            "INSERT INTO signatures (signer_address, transaction_id, signature) VALUES ($1, $2, $3);";
        let stmt = client.prepare(&_stmt).await?;
        client
            .execute(&stmt, &[signer_address, transaction_id, &signature])
            .await?;

        Ok(CkbTransaction {
            transaction_id: transaction_id.clone(),
            payload: payload.clone(),
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
        let stmt = client.prepare(&_stmt).await?;

        let row = client
            .query(&stmt, &[&multi_sig_address, &signer_address])
            .await?
            .pop();

        Ok(match row {
            Some(row) => Some(MultiSigSigner::from_row_ref(&row).unwrap()),
            None => None,
        })
    }

    pub async fn get_list_signatures_by_txid(
        &self,
        txid: &String,
    ) -> Result<Vec<CkbSignature>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM signatures 
            WHERE transaction_id=$1;";
        let stmt = client.prepare(&_stmt).await?;

        let signatures = client
            .query(&stmt, &[&txid])
            .await?
            .iter()
            .map(|row| CkbSignature::from_row_ref(&row).unwrap())
            .collect::<Vec<CkbSignature>>();

        Ok(signatures)
    }

    pub async fn sync_status_after_broadcast(
        &self,
        outpoints: Vec<String>,
        transaction_id: &String,
        payload: &String,
    ) -> Result<CkbTransaction, PoolError> {
        let mut client: Client = self.db.get().await?;

        let db_transaction = client.transaction().await?;

        // Update tx
        let _stmt = "UPDATE transactions SET status = 1, payload = $2  WHERE transaction_id = $1;";
        let stmt = db_transaction.prepare(&_stmt).await?;
        db_transaction
            .execute(&stmt, &[transaction_id, payload])
            .await?;

        // Update cells from tx info
        for outpoint in outpoints {
            let _stmt = "UPDATE cells SET status = 1 WHERE outpoint = $1 AND transaction_id = $2;";
            let stmt = db_transaction.prepare(&_stmt).await?;
            db_transaction
                .execute(&stmt, &[&outpoint, transaction_id])
                .await?;
        }

        db_transaction.commit().await?;

        Ok(CkbTransaction {
            transaction_id: transaction_id.clone(),
            payload: payload.clone(),
            status: 1,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
    }
}
