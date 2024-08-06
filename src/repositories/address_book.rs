use std::sync::Arc;

use crate::{models::address_book::AddressBook, serialize::address_book::AddressBookReq};
use chrono::Utc;
use deadpool_postgres::{Client, Pool, PoolError};
use tokio_pg_mapper::FromTokioPostgresRow;

#[derive(Clone, Debug)]
pub struct AddressBookDao {
    db: Arc<Pool>,
}

impl AddressBookDao {
    pub fn new(db: Arc<Pool>) -> Self {
        AddressBookDao { db: db.clone() }
    }

    pub async fn get_address_books(&self, address: &String) -> Result<Vec<AddressBook>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM address_books WHERE user_address=$1;";
        let stmt = client.prepare(_stmt).await?;

        let addresses = client
            .query(&stmt, &[&address])
            .await?
            .iter()
            .map(|row| AddressBook::from_row_ref(row).unwrap())
            .collect::<Vec<AddressBook>>();

        Ok(addresses)
    }

    pub async fn get_address(
        &self,
        address: &String,
        signer_address: &String,
    ) -> Result<Option<AddressBook>, PoolError> {
        let client: Client = self.db.get().await?;

        let stmt = "SELECT * FROM address_books WHERE user_address=$1 AND signer_address=$2;";
        match client.query_opt(stmt, &[address, signer_address]).await? {
            Some(row) => {
                let address_book = AddressBook::from_row_ref(&row).unwrap();
                Ok(Some(address_book))
            }
            None => Ok(None),
        }
    }

    pub async fn add_address(
        &self,
        address: &String,
        signer_address: &String,
        signer_name: &String,
    ) -> Result<AddressBook, PoolError> {
        let client: Client = self.db.get().await?;
        let stmt: &str = "INSERT INTO address_books (user_address, signer_address, signer_name) VALUES ($1, $2, $3);";
        client
            .execute(stmt, &[address, signer_address, signer_name])
            .await?;

        Ok(AddressBook {
            user_address: address.clone(),
            signer_address: signer_address.clone(),
            signer_name: signer_name.clone(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        })
    }

    pub async fn update_address(
        &self,
        user_address: &String,
        req: AddressBookReq,
    ) -> Result<bool, PoolError> {
        let client: Client = self.db.get().await?;
        let stmt =
            "UPDATE address_books SET signer_name=$1 WHERE user_address=$2 AND signer_address=$3";
        let res = client
            .execute(stmt, &[&req.signer_name, user_address, &req.signer_address])
            .await?;

        Ok(res > 0)
    }
}
