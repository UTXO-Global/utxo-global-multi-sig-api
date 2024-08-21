use crate::{
    config,
    handlers::{address_book, bounty_contest, ckb_explorer, multi_sig_account},
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
    let user_dao: repositories::user::UserDao = repositories::user::UserDao::new(db.clone());
    let multi_sig_dao = repositories::multi_sig_account::MultiSigDao::new(db.clone());
    let address_book_dao = repositories::address_book::AddressBookDao::new(db.clone());
    // TODO: @Broustail : 5
    // declare DAO object here
    let bounty_contest_dao = repositories::bounty_contest::BountyContestDao::new(db.clone());

    let user_service = web::Data::new(services::user::UserSrv::new(user_dao));
    let multi_sig_service = web::Data::new(services::multi_sig_account::MultiSigSrv::new(
        multi_sig_dao,
        address_book_dao.clone(),
    ));
    let address_book_service = web::Data::new(services::address_book::AddressBookSrv::new(
        address_book_dao.clone(),
    ));

    // TODO: @Broustail : 6
    // declare Service object here
    let bounty_contest_srv =
        services::bounty_contest::BountyContestSrv::new(bounty_contest_dao.clone());

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
            .app_data(web::Data::new(bounty_contest_srv.clone())) // TODO: @Broustail : 7 // pass service to handler via web_data
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(init_routes)
    })
    .bind(listen_address)?
    .run()
    .await
}
