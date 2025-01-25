use crate::{
    models::multi_sig_invite::MultiSigInviteStatus,
    serialize::{
        error::AppError,
        multi_sig_account::{
            InviteStatusReq, MultiSigAccountUpdateReq, NewMultiSigAccountReq, NewTransferReq,
            SubmitSignatureReq, TransactionFilters, UpdateTransactionStatusReq,
        },
    },
    services::multi_sig_account::MultiSigSrv,
};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use serde_json::json;

use super::jwt::JwtMiddleware;

async fn request_multi_sig_info(
    address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let signer = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };

    match multi_sig_srv
        .request_multi_sig_info_for_signer(&address, &signer)
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn request_list_signers(
    address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let signer = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };

    match multi_sig_srv.request_list_signers(&address, &signer).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn request_invites_list(
    multi_sig_srv: web::Data<MultiSigSrv>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };
    match multi_sig_srv.get_invites_list(&address).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn request_accept_invite(
    multisig_address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };
    let req = &InviteStatusReq {
        address,
        multisig_address: multisig_address.to_string(),
        status: MultiSigInviteStatus::ACCEPTED as i16,
    };
    match multi_sig_srv.update_invite_status(req.clone()).await {
        Ok(res) => Ok(HttpResponse::Ok().json(json!({"result": res}))),
        Err(err) => Err(err),
    }
}

async fn request_reject_invite(
    multisig_address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };
    let req = &InviteStatusReq {
        address,
        multisig_address: multisig_address.to_string(),
        status: MultiSigInviteStatus::REJECTED as i16,
    };

    match multi_sig_srv.update_invite_status(req.clone()).await {
        Ok(res) => Ok(HttpResponse::Ok().json(json!({"result": res}))),
        Err(err) => Err(err),
    }
}

async fn request_list_accounts(
    multi_sig_srv: web::Data<MultiSigSrv>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let signer_address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };
    match multi_sig_srv.request_list_accounts(&signer_address).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn request_list_transactions(
    filters: web::Query<TransactionFilters>,
    multisig_address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let user_address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };
    match multi_sig_srv
        .request_list_transactions(&user_address, &multisig_address, filters.into_inner())
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn create_new_account(
    multi_sig_srv: web::Data<MultiSigSrv>,
    req: web::Json<NewMultiSigAccountReq>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let user_address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };
    match multi_sig_srv
        .create_new_account(&user_address, req.clone())
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn request_update_account(
    multi_sig_srv: web::Data<MultiSigSrv>,
    req: web::Json<MultiSigAccountUpdateReq>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let user_address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };

    match multi_sig_srv
        .update_account(&user_address, req.clone())
        .await
    {
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
    let user_address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };

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
    let user_address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };

    match multi_sig_srv
        .submit_signature(&user_address, &req.signature, &req.txid)
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn reject_transaction(
    multi_sig_srv: web::Data<MultiSigSrv>,
    transaction_id: web::Path<String>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let user_address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };

    match multi_sig_srv
        .reject_transaction(&user_address, &transaction_id)
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(json!({ "result": res }))),
        Err(err) => Err(err),
    }
}

async fn request_transaction_summary(
    multisig_address: web::Path<String>,
    multi_sig_srv: web::Data<MultiSigSrv>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let user_address = {
        let ext = http_req.extensions();
        ext.get::<String>().unwrap().clone()
    };
    match multi_sig_srv
        .rp_transaction_summary(&user_address, &multisig_address)
        .await
    {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

async fn update_transaction_commited(
    req: web::Json<UpdateTransactionStatusReq>,
    multi_sig_srv: web::Data<MultiSigSrv>,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    match multi_sig_srv.update_transaction_commited(&req).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

pub fn route(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/multi-sig")
            .route("/info/{address}", web::get().to(request_multi_sig_info))
            .route("/signers/{address}", web::get().to(request_list_signers))
            .route("/invites", web::get().to(request_invites_list))
            .route(
                "/invites/accept/{address}",
                web::put().to(request_accept_invite),
            )
            .route(
                "/invites/reject/{address}",
                web::put().to(request_reject_invite),
            )
            .route("/accounts", web::get().to(request_list_accounts))
            .route("/accounts", web::put().to(request_update_account))
            .route(
                "/transactions/{address}",
                web::get().to(request_list_transactions),
            )
            .route(
                "/transactions/{address}/commited",
                web::put().to(update_transaction_commited),
            )
            .route(
                "/transactions/{address}/summary",
                web::get().to(request_transaction_summary),
            )
            .route(
                "/transactions/{txId}/reject",
                web::put().to(reject_transaction),
            )
            .route("/new-transfer", web::post().to(create_new_transfer))
            .route("/signature", web::post().to(submit_signature))
            .route("/new-account", web::post().to(create_new_account)),
    );
}
