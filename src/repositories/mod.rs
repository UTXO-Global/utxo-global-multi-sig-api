use crate::config;
use deadpool_postgres::tokio_postgres::NoTls;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static DB_POOL: Lazy<Arc<Pool>> = Lazy::new(|| {
    let mut cfg = Config::new();
    let database_url: String = config::get("database_url");
    cfg.url = Some(database_url);
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = cfg.create_pool(None, NoTls).unwrap();

    Arc::new(pool)
});

pub mod multi_sig_account;
pub mod user;
