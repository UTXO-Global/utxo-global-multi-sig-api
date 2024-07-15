use chrono::Utc;
use ethers::{prelude::*, utils::hex};
use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;

use crate::{
    config,
    models::user::User,
    repositories::user::UserDao,
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

    pub async fn get_nonce(&self, address: &String) -> Result<UserRequestNonceRes, AppError> {
        let address = address.to_lowercase();
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

    async fn verify_signature(&self, req: LoginReq) -> Result<User, AppError> {
        match self
            .user_dao
            .get_user_by_address(&req.address.to_lowercase())
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))?
        {
            Some(user) => {
                let nonce = user.nonce.clone().unwrap();

                // Update a new nonce for user
                let _ = self.update_user_nonce(user.clone());

                let signature = req.signature;
                let message = format!("\x19Ethereum Signed Message:\n{}{}", nonce.len(), nonce);
                let message_hash = ethers::utils::keccak256(message.clone().as_bytes());

                let sig_bytes = hex::decode(&signature[2..]).expect("Failed to decode signature");
                let sig =
                    Signature::try_from(sig_bytes.as_slice()).expect("Failed to parse signature");

                match sig.recover(message_hash) {
                    Ok(recovered) => {
                        if recovered == req.address.to_lowercase().parse::<Address>().unwrap() {
                            Ok(user.clone())
                        } else {
                            Err(AppError::new(500).message("Signature not matched"))
                        }
                    }
                    Err(err) => Err(AppError::new(500).message(&err.to_string())),
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
                    sub: req.address.to_lowercase(),
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
            .update_user(&user.user_address.to_lowercase(), user.clone())
            .await;
        return res.is_ok();
    }
}
