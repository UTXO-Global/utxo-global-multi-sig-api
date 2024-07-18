use crate::{
    serialize::{
        error::AppError,
        multi_sig_account::{
            NewMultiSigAccountReq, NewTransferReq, SubmitSignatureReq, TransactionFilters,
        },
    },
    services::multi_sig_account::MultiSigSrv,
};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};

use super::jwt::JwtMiddleware;

async fn request_multi_sig_info(
    address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
) -> Result<HttpResponse, AppError> {
    match multi_sig_srv.request_multi_sig_info(&address).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn request_list_signers(
    address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
) -> Result<HttpResponse, AppError> {
    match multi_sig_srv.request_list_signers(&address).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn request_list_accounts(
    signer_address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
) -> Result<HttpResponse, AppError> {
    match multi_sig_srv.request_list_accounts(&signer_address).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn request_list_transactions(
    query: web::Query<TransactionFilters>,
    signer_address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
) -> Result<HttpResponse, AppError> {
    match multi_sig_srv
        .request_list_transactions(
            &signer_address,
            query.offset.unwrap_or(0),
            query.limit.unwrap_or(10),
        )
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn create_new_account(
    multi_sig_srv: web::Data<MultiSigSrv>,
    req: web::Json<NewMultiSigAccountReq>,
) -> Result<HttpResponse, AppError> {
    match multi_sig_srv.create_new_account(req.clone()).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn create_new_transfer(
    multi_sig_srv: web::Data<MultiSigSrv>,
    req: web::Json<NewTransferReq>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let ext = http_req.extensions();
    let user_address = ext.get::<String>().unwrap();

    match multi_sig_srv
        .create_new_transfer(&user_address, &req.signature, &req.payload)
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn submit_signature(
    multi_sig_srv: web::Data<MultiSigSrv>,
    req: web::Json<SubmitSignatureReq>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let ext = http_req.extensions();
    let user_address = ext.get::<String>().unwrap();

    match multi_sig_srv
        .submit_signature(&user_address, &req.signature, &req.txid)
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

pub fn route(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/multi-sig")
            .route("/info/{address}", web::get().to(request_multi_sig_info))
            .route("/list/{address}", web::get().to(request_list_signers))
            .route("/accounts/{address}", web::get().to(request_list_accounts))
            .route(
                "/transactions/{address}",
                web::get().to(request_list_transactions),
            )
            .route("/new-transfer", web::post().to(create_new_transfer))
            .route("/signature", web::post().to(submit_signature))
            .route("/new-account", web::post().to(create_new_account)),
    );
}
