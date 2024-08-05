use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SignerInfo {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewMultiSigAccountReq {
    pub name: String,
    pub threshold: i16,
    pub signers: Vec<SignerInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewTransferReq {
    pub signature: String,
    pub payload: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubmitSignatureReq {
    pub signature: String,
    pub txid: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransactionFilters {
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}
