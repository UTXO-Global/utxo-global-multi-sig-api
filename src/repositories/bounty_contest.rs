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
}
