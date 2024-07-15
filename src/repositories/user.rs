use std::sync::Arc;

use crate::models::user::User;
use deadpool_postgres::{Client, Pool, PoolError};
use tokio_pg_mapper::FromTokioPostgresRow;

#[derive(Clone, Debug)]
pub struct UserDao {
    db: Arc<Pool>,
}

impl UserDao {
    pub fn new(db: Arc<Pool>) -> Self {
        UserDao { db: db.clone() }
    }

    pub async fn get_user_by_address(&self, address: &String) -> Result<Option<User>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM users 
            WHERE user_address=$1;";
        let stmt = client.prepare(&_stmt).await?;

        let row = client.query(&stmt, &[&address]).await?.pop();

        Ok(match row {
            Some(row) => Some(User::from_row_ref(&row).unwrap()),
            None => None,
        })
    }

    pub async fn add_user(&self, user: User) -> Result<User, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "INSERT INTO users (user_address, nonce) VALUES ($1, $2);";
        let stmt = client.prepare(&_stmt).await?;

        client
            .execute(&stmt, &[&user.user_address, &user.nonce])
            .await?;
        Ok(user)
    }

    pub async fn update_user(&self, user_address: &String, user: User) -> Result<User, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "UPDATE users SET nonce = $2, updated_at = NOW() WHERE user_address = $1;";
        let stmt = client.prepare(&_stmt).await?;

        client.execute(&stmt, &[&user_address, &user.nonce]).await?;
        Ok(user)
    }
}
