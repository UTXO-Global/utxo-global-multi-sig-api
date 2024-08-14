use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

pub enum TransactionStatus {
    Peinding,
    Sent,
    Rejected,
    Failed,
}

pub const TRANSACTION_STATUS_PENDING: i16 = TransactionStatus::Peinding as i16;
pub const TRANSACTION_STATUS_SENT: i16 = TransactionStatus::Sent as i16;
pub const TRANSACTION_STATUS_REJECT: i16 = TransactionStatus::Rejected as i16;
pub const TRANSACTION_STATUS_FAILED: i16 = TransactionStatus::Failed as i16;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "transactions")]
pub struct CkbTransaction {
    pub transaction_id: String,
    pub multi_sig_address: String,
    pub payload: String,
    pub status: i16,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "signatures")]
pub struct CkbSignature {
    pub signer_address: String,
    pub transaction_id: String,
    pub signature: String,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "transaction_errors")]
pub struct TransactionError {
    pub transaction_id: String,
    pub signer_address: String,
    pub errors: String,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "transaction_rejects")]
pub struct TransactionReject {
    pub transaction_id: String,
    pub signer_address: String,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}
