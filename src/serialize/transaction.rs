use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionInfo {
    pub transaction_id: String,
    pub multi_sig_address: String,
    pub to_address: String,
    pub confirmed: Vec<String>,
    pub status: i16,
    pub payload: String,
    pub created_at: String,
    pub amount: u64,
}
