use std::sync::Arc;

use crate::{
    models::{
        multi_sig_account::{MultiSigInfo, MultiSigSigner},
        multi_sig_invite::MultiSigInvite,
        multi_sig_tx::{CkbSignature, CkbTransaction},
    },
    serialize::multi_sig_account::{MultiSigAccountUpdateReq, NewMultiSigAccountReq},
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

    pub async fn request_list_signers(
        &self,
        address: &String,
    ) -> Result<Vec<MultiSigSigner>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM multi_sig_signers 
            WHERE multi_sig_address=$1;";
        let stmt = client.prepare(_stmt).await?;

        let signers = client
            .query(&stmt, &[&address])
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
        signer_address: &String,
        multisig_address: &String,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<CkbTransaction>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT tx.* FROM transactions tx
            LEFT JOIN multi_sig_signers mss
                ON mss.multi_sig_address = tx.multi_sig_address
            WHERE mss.signer_address=$1 and tx.multi_sig_address = $2
            OFFSET $3 LIMIT $4;";
        let stmt = client.prepare(_stmt).await?;

        let txs = client
            .query(
                &stmt,
                &[
                    &signer_address,
                    &multisig_address,
                    &(offset as i64),
                    &(limit as i64),
                ],
            )
            .await?
            .iter()
            .map(|row| CkbTransaction::from_row_ref(row).unwrap())
            .collect::<Vec<CkbTransaction>>();

        Ok(txs)
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
}
