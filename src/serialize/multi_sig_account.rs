use serde::{Deserialize, Serialize};

use crate::models::{multi_sig_account::MultiSigSigner, multi_sig_invite::MultiSigInvite};

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
pub struct MultiSigAccountUpdateReq {
    pub multi_sig_address: String,
    pub name: String,
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

#[derive(Debug, Deserialize, Clone)]
pub struct InviteStatusReq {
    pub address: String,
    pub multisig_address: String,
    pub status: i16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InviteInfo {
    pub address: String,
    pub multisig_address: String,
    pub account_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListSignerRes {
    pub signers: Vec<MultiSigSigner>,
    pub invites: Vec<MultiSigInvite>,
}
