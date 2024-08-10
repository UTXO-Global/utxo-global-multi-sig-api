use std::io;
use utxo_global_multi_sig_api::app;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> io::Result<()> {
    app::create_app().await
}
