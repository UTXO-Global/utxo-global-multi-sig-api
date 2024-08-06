use std::str::FromStr;

use chrono::Utc;

use ckb_hash::{Blake2bBuilder, CKB_HASH_PERSONALIZATION};
use ckb_sdk::{Address, AddressPayload};
use ckb_types::packed::Script;
use jsonwebtoken::{encode, EncodingKey, Header};
use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, PublicKey, Secp256k1,
};
use uuid::Uuid;

use crate::{
    config,
    models::user::User,
    repositories::{ckb::get_ckb_network, user::UserDao},
    serialize::{
        error::AppError,
        user::{LoginReq, LoginRes, UserRequestNonceRes},
        Claims,
    },
};

#[derive(Clone, Debug)]
pub struct UserSrv {
    user_dao: UserDao,
}

impl UserSrv {
    pub fn new(user_dao: UserDao) -> Self {
        UserSrv {
            user_dao: user_dao.clone(),
        }
    }

    fn hash_ckb(&self, message: &[u8]) -> [u8; 32] {
        let mut hasher = Blake2bBuilder::new(32)
            .personal(CKB_HASH_PERSONALIZATION)
            .build();
        hasher.update(message);
        let mut result = [0; 32];
        hasher.finalize(&mut result);
        result
    }

    pub async fn get_nonce(&self, address: &String) -> Result<UserRequestNonceRes, AppError> {
        match Address::from_str(address) {
            Ok(_) => {
                match self
                    .user_dao
                    .get_user_by_address(&address.clone())
                    .await
                    .map_err(|err| AppError::new(500).message(&err.to_string()))?
                {
                    Some(mut user) => {
                        if user.nonce.is_none() {
                            user.nonce = Some(Uuid::new_v4().to_string());
                            let _ = match self.user_dao.update_user(&address, user.clone()).await {
                                Ok(u) => Ok(UserRequestNonceRes {
                                    nonce: u.nonce.unwrap(),
                                    address: address.to_string(),
                                }),
                                Err(err) => Err(AppError::new(500).message(&err.to_string())),
                            };
                        }

                        Ok(UserRequestNonceRes {
                            nonce: user.nonce.clone().unwrap(),
                            address: address.to_string(),
                        })
                    }
                    None => {
                        let user_default = User::default(address.to_string());
                        match self.user_dao.add_user(user_default).await {
                            Ok(user) => Ok(UserRequestNonceRes {
                                nonce: user.nonce.unwrap(),
                                address: address.to_string(),
                            }),
                            Err(err) => Err(AppError::new(500).message(&err.to_string())),
                        }
                    }
                }
            }
            Err(err) => {
                return Err(AppError::new(404).message(&format!("invalid address: {}", err)));
            }
        }
    }

    async fn verify_signature(&self, req: LoginReq) -> Result<User, AppError> {
        match self
            .user_dao
            .get_user_by_address(&req.address)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?
        {
            Some(user) => {
                let nonce = user.nonce.clone().unwrap();

                // Update a new nonce for user
                let _ = self.update_user_nonce(user.clone()).await;

                let signature = req.signature;
                let message = format!("Nervos Message:utxo.global login {}", nonce);
                let message_hash = self.hash_ckb(message.as_bytes());
                let secp_message =
                    Message::from_slice(&message_hash).expect("Invalid message hash");

                let sig_bytes = hex::decode(&signature).expect("Invalid signature hex");
                let r = &sig_bytes[0..32];
                let s = &sig_bytes[32..64];
                let rec_id = sig_bytes[64]; // Recovery ID as byte
                let rec_id = RecoveryId::from_i32(rec_id as i32).expect("Invalid recovery ID");
                let mut ret: [u8; 64] = [0; 64];
                ret[..32].copy_from_slice(r);
                ret[32..].copy_from_slice(s);

                let rec_sig = RecoverableSignature::from_compact(&ret, rec_id)
                    .expect("Invalid recoverable signature");

                let secp = Secp256k1::new();
                let pub_key = secp
                    .recover_ecdsa(&secp_message, &rec_sig)
                    .expect("Failed to recover public key");

                let pub_key_bytes = pub_key.serialize();
                let expected_pubkey =
                    PublicKey::from_slice(&pub_key_bytes).expect("Invalid public key");
                let address = Address::from_str(&req.address).unwrap();
                let recovered_address = Address::new(
                    get_ckb_network(),
                    AddressPayload::from_pubkey(&expected_pubkey),
                    true,
                );

                if recovered_address.to_string() == address.to_string() {
                    Ok(user.clone())
                } else {
                    Err(AppError::new(500).message("Signature not matched"))
                }
            }
            None => Err(AppError::new(404).message("no user found")),
        }
    }

    pub async fn login(&self, req: LoginReq) -> Result<LoginRes, AppError> {
        match self.verify_signature(req.clone()).await {
            Ok(_user) => {
                // create jwt
                let expired_time = (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize;
                let my_claims = Claims {
                    sub: req.address,
                    exp: expired_time,
                    aud: false,
                    iat: Utc::now().timestamp() as usize,
                };
                let jwt_secret: String = config::get("jwt_secret");
                let jwt = encode(
                    &Header::default(),
                    &my_claims,
                    &EncodingKey::from_secret(jwt_secret.as_ref()),
                )
                .unwrap();
                Ok(LoginRes {
                    token: jwt,
                    expired: expired_time,
                })
            }
            Err(err) => Err(err),
        }
    }

    async fn update_user_nonce(&self, mut user: User) -> bool {
        user.nonce = Some(Uuid::new_v4().to_string());
        let res = self
            .user_dao
            .update_user(&user.user_address, user.clone())
            .await;
        return res.is_ok();
    }
}
