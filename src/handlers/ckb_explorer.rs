use actix_web::{web, HttpResponse};
use reqwest::{header, Client};
use serde_json::Value;

use crate::{config, serialize::error::AppError};

// Proxy request đến server đích
async fn proxy_request(
    method: &str,
    path: &str,
    body: Option<String>,
) -> Result<HttpResponse, AppError> {
    let ckb_api_url: String = config::get("ckb_explorer_api");
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
        .header(header::CONTENT_TYPE, "application/vnd.api+json");

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

async fn ckb_handle_get_request(path: web::Path<String>) -> Result<HttpResponse, AppError> {
    proxy_request("GET", &path, None).await
}

async fn ckb_handle_post_request(
    req_body: web::Json<Value>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let body = Some(req_body.to_string());
    proxy_request("POST", &path, body).await
}

async fn ckb_handle_put_request(
    req_body: web::Json<Value>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let body = Some(req_body.to_string());
    proxy_request("POST", &path, body).await
}

pub fn route(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/ckb")
            .route("/{path:.*}", web::get().to(ckb_handle_get_request))
            .route("/{path:.*}", web::post().to(ckb_handle_post_request))
            .route("/{path:.*}", web::put().to(ckb_handle_put_request)),
    );
}
