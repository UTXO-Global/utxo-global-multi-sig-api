use crate::{
    config,
    handlers::{address_book, ckb_explorer, multi_sig_account},
    repositories::{self, db::DB_POOL},
    services,
};
use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};

use crate::handlers::user;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    user::route(cfg);
    multi_sig_account::route(cfg);
    address_book::route(cfg);
    ckb_explorer::route(cfg);
}

pub async fn create_app() -> std::io::Result<()> {
    // Init DB
    let db = &DB_POOL.clone();
    let user_dao = repositories::user::UserDao::new(db.clone());
    let multi_sig_dao = repositories::multi_sig_account::MultiSigDao::new(db.clone());
    let address_book_dao = repositories::address_book::AddressBookDao::new(db.clone());
    let user_service = web::Data::new(services::user::UserSrv::new(user_dao));
    let multi_sig_service = web::Data::new(services::multi_sig_account::MultiSigSrv::new(
        multi_sig_dao,
        address_book_dao.clone(),
    ));
    let address_book_service = web::Data::new(services::address_book::AddressBookSrv::new(
        address_book_dao.clone(),
    ));

    let listen_address: String = config::get("listen_address");

    println!("\nListening and serving HTTP on {}", listen_address);

    HttpServer::new(move || {
        let cors: Cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
            .max_age(3600);

        App::new()
            .app_data(user_service.clone())
            .app_data(multi_sig_service.clone())
            .app_data(address_book_service.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(init_routes)
    })
    .bind(listen_address)?
    .run()
    .await
}
