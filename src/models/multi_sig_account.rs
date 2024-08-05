use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

pub enum MultiSigSingerStatus {
    PENDING,
    ACCEPTED,
    REJECTED,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "multi_sig_signers")]
pub struct MultiSigSigner {
    pub multi_sig_address: String,
    pub signer_address: String,
    pub name: String,
    pub status: i16,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "multi_sig_info")]
pub struct MultiSigInfo {
    pub multi_sig_address: String,
    pub threshold: i16,
    pub signers: i16,
    pub name: String,
    pub mutli_sig_witness_data: String,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}
