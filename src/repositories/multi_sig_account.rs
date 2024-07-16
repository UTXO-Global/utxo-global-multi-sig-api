use std::sync::Arc;

use crate::{
    models::multi_sig_account::{MultiSigInfo, MultiSigSigner},
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

    pub async fn create_new_account(
        &self,
        multi_sig_address: &String,
        req: &NewMultiSigAccountReq,
    ) -> Result<MultiSigInfo, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "INSERT INTO multi_sig_info (multi_sig_address, threshold, signers, name) VALUES ($1, $2, $3, $4);";
        let stmt = client.prepare(&_stmt).await?;

        client
            .execute(
                &stmt,
                &[
                    multi_sig_address,
                    &req.threshold,
                    &(req.signers.len() as i16),
                    &req.name,
                ],
            )
            .await?;
        Ok(MultiSigInfo {
            multi_sig_address: multi_sig_address.clone(),
            threshold: req.threshold,
            signers: req.signers.len() as i16,
            name: req.name.clone(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
    }
}
