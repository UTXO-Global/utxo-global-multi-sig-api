use crate::{config, repositories, services};
use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};

use crate::handlers::user;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    user::route(cfg);
}

pub async fn create_app() -> std::io::Result<()> {
    // Init DB
    let db = &repositories::DB_POOL.clone();
    let user_dao = repositories::user::UserDao::new(db.clone());
    let user_service = web::Data::new(services::user::UserSrv::new(user_dao));

    let listen_address: String = config::get("listen_address");

    println!("\nListening and serving HTTP on {}", listen_address);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
            .max_age(3600);

        App::new()
            .app_data(user_service.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(init_routes)
    })
    .bind(listen_address)?
    .run()
    .await
}
