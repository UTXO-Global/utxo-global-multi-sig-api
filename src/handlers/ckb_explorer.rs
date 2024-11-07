use actix_web::{web, HttpRequest, HttpResponse};
use ckb_sdk::NetworkType;
use reqwest::{header, Client};
use serde_json::Value;

use crate::{repositories::ckb::get_explorer_api_url, serialize::error::AppError};

async fn proxy_request(
    method: &str,
    network: &str,
    path: &str,
    body: Option<String>,
) -> Result<HttpResponse, AppError> {
    let ckb_api_url: String = if network == "testnet" {
        get_explorer_api_url(NetworkType::Testnet)
    } else {
        get_explorer_api_url(NetworkType::Mainnet)
    };
    let endpoint: String = format!("{}/{}", ckb_api_url, path.to_owned());
    let client = Client::new();
    let mut request_builder = match method {
        "GET" => client.get(endpoint),
        "POST" => client.post(endpoint),
        "PUT" => client.put(endpoint),
        _ => return Err(AppError::new(500).message("Method not allowed")),
    };

    request_builder = request_builder
        .header(header::ACCEPT, "application/vnd.api+json")
        .header(header::CONTENT_TYPE, "application/vnd.api+json")
        .header(header::USER_AGENT, "curl/7.68.0");

    if let Some(body) = body {
        request_builder = request_builder.body(body);
    }

    let response = request_builder
        .send()
        .await
        .map_err(|error| AppError::new(500).message(&error.to_string()))?;

    let result: Value = response
        .json()
        .await
        .map_err(|error| AppError::new(500).message(&error.to_string()))?;

    Ok(HttpResponse::Ok().json(result))
}

async fn ckb_handle_get_request(
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (network, url) = path.into_inner();
    let url_and_query = format!("{}?{}", url, req.query_string());
    proxy_request("GET", &network, &url_and_query, None).await
}

async fn ckb_handle_post_request(
    req_body: web::Json<Value>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (network, url) = path.into_inner();
    let url_and_query = format!("{}?{}", url, req.query_string());
    let body = Some(req_body.to_string());
    proxy_request("POST", &network, &url_and_query, body).await
}

async fn ckb_handle_put_request(
    req_body: web::Json<Value>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let (network, url) = path.into_inner();
    let url_and_query = format!("{}?{}", url, req.query_string());
    let body = Some(req_body.to_string());
    proxy_request("POST", &network, &url_and_query, body).await
}

pub fn route(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/ckb")
            .route(
                "/{network}/{path:.*}",
                web::get().to(ckb_handle_get_request),
            )
            .route(
                "/{network}/{path:.*}",
                web::post().to(ckb_handle_post_request),
            )
            .route(
                "/{network}/{path:.*}",
                web::put().to(ckb_handle_put_request),
            ),
    );
}
