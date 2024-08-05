use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "address_books")]
pub struct AddressBook {
    pub user_address: String,
    pub signer_name: String,
    pub signer_address: String,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}
