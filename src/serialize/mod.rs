use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub aud: bool,
    pub iat: usize,
    pub exp: usize,
}

pub mod error;
pub mod user;
