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

// TODO: @Broustail : 3
// Define request you want to receive from client
// Define response you want to response to client
// You can create new file for define your struct

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaginationReq {
    // TODO: @Broustail : Example define PaginationReq to receive params from request
    pub page: i16,
    pub limit: i16,
}

pub mod address_book;
pub mod error;
pub mod multi_sig_account;
pub mod transaction;
pub mod user;
