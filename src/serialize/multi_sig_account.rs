use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct NewMultiSigAccountReq {
    pub name: String,
    pub threshold: i16,
    pub signers: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewTransferReq {
    pub signatures: Vec<String>,
    pub payload: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubmitSignatureReq {
    pub signatures: Vec<String>,
    pub txid: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransactionFilters {
    pub offset: Option<i32>,
    pub limit: Option<i32>,
}
