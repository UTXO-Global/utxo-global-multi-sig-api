use deadpool_postgres::{Client, Pool, PoolError};
use std::sync::Arc;
use tokio_pg_mapper::FromTokioPostgresRow;

use crate::models::bounty_contest::BountyContest;

#[derive(Clone, Debug)]
pub struct BountyContestDao {
    db: Arc<Pool>,
}

impl BountyContestDao {
    pub fn new(db: Arc<Pool>) -> Self {
        BountyContestDao { db: db.clone() }
    }

    // TODO: @Broustail : 2
    // Define your fn to connect to db

    pub async fn get_dashboard(
        &self,
        page: i16,
        limit: i16,
    ) -> Result<Vec<BountyContest>, PoolError> {
        // TODO: @Broustail: Example get rankings
        let offset = (page - 1) * limit;
        let client: Client = self.db.get().await?;

        let stmt = "SELECT * FROM RANKINGS ORDER BY Points DESC LIMIT $1 OFFSET $2 ";

        let results = client
            .query(stmt, &[&page, &offset])
            .await?
            .iter()
            .map(|row| BountyContest::from_row_ref(row).unwrap())
            .collect::<Vec<BountyContest>>();

        Ok(results)
    }
}
