use serde::{Deserialize, Serialize};

use super::PaginationRes;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionInfo {
    pub transaction_id: String,
    pub multi_sig_address: String,
    pub to_address: String,
    pub confirmed: Vec<String>,
    pub status: i16,
    pub payload: String,
    pub created_at: i64,
    pub amount: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionSumary {
    pub total_tx_pending: u32,
    pub total_amount_pending: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListTransactionsRes {
    pub transactions: Vec<TransactionInfo>,
    pub pagination: PaginationRes,
}
