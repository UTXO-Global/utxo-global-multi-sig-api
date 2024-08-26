use crate::models::bounty_contest::BountyContestLeaderboard;
use deadpool_postgres::{Client, Pool, PoolError};
use std::sync::Arc;
use tokio_pg_mapper::FromTokioPostgresRow;

#[derive(Clone, Debug)]
pub struct BountyContestDao {
    db: Arc<Pool>,
}

impl BountyContestDao {
    pub fn new(db: Arc<Pool>) -> Self {
        BountyContestDao { db: db.clone() }
    }
    pub async fn get_by_email(
        &self,
        email: &String,
    ) -> Result<Option<BountyContestLeaderboard>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM RANKINGS WHERE email=$1;";
        match client.query_opt(_stmt, &[email]).await? {
            Some(row) => {
                let bc = BountyContestLeaderboard::from_row_ref(&row).unwrap();
                Ok(Some(bc))
            }
            None => Ok(None),
        }
    }

    pub async fn get_dashboard(
        &self,
        page: i16,
        limit: i16,
    ) -> Result<Vec<BountyContestLeaderboard>, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "SELECT * FROM RANKINGS ORDER BY Points DESC LIMIT $1 OFFSET $2;";
        let stmt = client.prepare(_stmt).await?;

        let results = client
            .query(&stmt, &[&page, &limit])
            .await?
            .iter()
            .map(|row| BountyContestLeaderboard::from_row_ref(row).unwrap())
            .collect::<Vec<BountyContestLeaderboard>>();

        Ok(results)
    }

    pub async fn update_bc(&self, email: &String, points: i32) -> Result<bool, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "UPDATE bounty_contest_leaderboard SET points = $1 WHERE email = $2, updated_at = NOW()";
        let stmt = client.prepare(_stmt).await?;
        let res = client.execute(&stmt, &[&points, &email]).await?;
        Ok(res > 0)
    }

    pub async fn insert_bc(
        &self,
        email: &String,
        username: &String,
        points: i32,
    ) -> Result<bool, PoolError> {
        let client: Client = self.db.get().await?;

        let _stmt = "INSERT INTO RANKINGS (email, username, points) VALUES ($1, $2, $3), updated_at = NOW()";
        let stmt = client.prepare(_stmt).await?;
        let res = client.execute(&stmt, &[&email, &username, &points]).await?;
        Ok(res > 0)
    }

    pub async fn get_count(&self) -> Result<i64, PoolError> {
        let client: Client = self.db.get().await?;
        let _stmt = "SELECT count(*) FROM rankings";
        match client.query_opt(_stmt, &[]).await? {
            Some(row) => {
                let total_items: i64 = row.get(0);
                Ok(total_items)
            }
            None => Ok(0),
        }
    }
}
