use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserRequestNonceRes {
    pub address: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoginReq {
    pub signature: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRes {
    pub token: String,
    pub expired: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignatureRes {
    pub signature: String,
    pub nonce: String,
}
