use std::io;

use utxo_global_multi_sig_api::app;

#[actix_web::main]
async fn main() -> io::Result<()> {
    app::create_app().await
}
