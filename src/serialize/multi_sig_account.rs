use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct NewMultiSigAccountReq {
    pub name: String,
    pub threshold: i16,
    pub signers: Vec<String>,
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
