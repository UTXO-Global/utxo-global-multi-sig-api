use crate::{
    serialize::{error::AppError, user::LoginReq},
    services::user::UserSrv,
};
use actix_web::{
    cookie::{time::Duration as ActixWebDuration, Cookie},
    web, HttpResponse,
};

async fn request_nonce(
    address: web::Path<String>,
    user_srv: web::Data<UserSrv>,
) -> Result<HttpResponse, AppError> {
    match user_srv.get_nonce(&address).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn login(
    user_srv: web::Data<UserSrv>,
    req: web::Json<LoginReq>,
) -> Result<HttpResponse, AppError> {
    match user_srv.login(req.clone()).await {
        Ok(res) => {
            let cookie = Cookie::build("utxo-global-multi-sig_cookie", res.clone().token)
                .path("/")
                .max_age(ActixWebDuration::new(60 * 60, 0))
                .http_only(true)
                .finish();

            Ok(HttpResponse::Ok().cookie(cookie).json(res))
        }
        Err(err) => Err(err),
    }
}

pub fn route(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/users")
            .route("/nonce/{address}", web::get().to(request_nonce))
            .route("/login", web::post().to(login)),
    );
}
