use std::sync::Arc;

use utxo_global_multi_sig_api::{
    repositories::{
        db::{migrate_db, DB_POOL},
        multi_sig_account::MultiSigDao,
    },
    services::multi_sig_account::MultiSigSrv,
};

async fn run_crons(multi_sig_service: Arc<MultiSigSrv>) {
    // let time_duration: u64 = 10;
    // loop {
    //     let _ = multi_sig_service.sync_ckb_status().await;
    //     thread::sleep(Duration::from_secs(time_duration));
    // }

    let _ = multi_sig_service.sync_ckb_status().await;
}

#[tokio::main]
async fn main() {
    let db = &DB_POOL.clone();

    if let Err(e) = migrate_db().await {
        println!("\nMigrate db failed: {e}");
    }

    let multi_sig_dao = MultiSigDao::new(db.clone());
    let multi_sig_service = Arc::new(MultiSigSrv::new(multi_sig_dao));

    println!("Crons is running...");
    run_crons(multi_sig_service).await
}
