use crate::{serialize::error::AppError, services::address_book::AddressBookSrv};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};

use super::jwt::JwtMiddleware;

async fn request_get_address_books(
    address_book_srv: web::Data<AddressBookSrv>,
    http_req: HttpRequest,
    _: JwtMiddleware,
) -> Result<HttpResponse, AppError> {
    let ext = http_req.extensions();
    let address = ext.get::<String>().unwrap().to_string();
    match address_book_srv.get_address_list(&address).await {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
    }
}

pub fn route(conf: &mut web::ServiceConfig) {
    conf.service(web::scope("/address-books").route("/", web::get().to(request_get_address_books)));
}
