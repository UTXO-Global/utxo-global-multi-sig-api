use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub aud: bool,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaginationRes {
    pub page: i64,
    pub limit: i64,
    pub total_records: i64,
    pub total_page: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaginationReq {
    pub page: i64,
    pub limit: i64,
}

pub mod address_book;
pub mod bounty_contest;
pub mod error;
pub mod multi_sig_account;
pub mod transaction;
pub mod user;
