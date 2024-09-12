use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;
pub enum MultiSigInviteStatus {
    PENDING,
    ACCEPTED,
    REJECTED,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "multi_sig_invites")]
pub struct MultiSigInvite {
    pub multi_sig_address: String,
    pub signer_address: String,
    pub status: i16,

    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,

    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}
