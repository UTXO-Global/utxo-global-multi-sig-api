use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, FromRequest, HttpMessage, HttpRequest};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::future::{ready, Ready};

use crate::config;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub aud: bool,
    pub iat: usize,
    pub exp: usize,
}

pub struct JwtMiddleware {
    pub address: String,
}

impl FromRequest for JwtMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let jwt_secret: String = config::get("jwt_secret");

        let token = req
            .cookie("utxo-global-multi-sig_cookie")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        if token.is_none() {
            return ready(Err(ErrorUnauthorized(
                json!({"message": "You are not logged in, please provide token"}),
            )));
        }

        let claims = match decode::<TokenClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(_) => {
                return ready(Err(ErrorUnauthorized(json!({"message": "Invalid token"}))));
            }
        };

        let address = claims.sub;
        req.extensions_mut().insert::<String>(address.to_owned());

        ready(Ok(JwtMiddleware { address }))
    }
}
