use chrono::{NaiveDateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "users")]
pub struct User {
    pub user_address: String,
    pub nonce: Option<String>,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn default(user_address: String) -> User {
        let nonce = Uuid::new_v4();
        User {
            user_address,
            nonce: Some(nonce.to_string()),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}
