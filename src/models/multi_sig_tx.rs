use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "cells")]
pub struct CkbCell {
    pub multi_sig_address: String,
    pub outpoint: String,
    pub transaction_id: String,
    pub status: i16,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "transactions")]
pub struct CkbTransaction {
    pub transaction_id: String,
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
