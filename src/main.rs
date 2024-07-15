use std::io;

mod app;
mod config;
mod handlers;
mod models;
mod repositories;
mod serialize;
mod services;

#[actix_web::main]
async fn main() -> io::Result<()> {
    app::create_app().await
}
