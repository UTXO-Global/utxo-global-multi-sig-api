use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AddressBookReq {
    pub signer_address: String,
    pub signer_name: String,
}
