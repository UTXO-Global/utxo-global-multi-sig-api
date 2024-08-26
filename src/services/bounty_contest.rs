use crate::{
    models::bounty_contest::BountyContestLeaderboard,
    repositories::bounty_contest::BountyContestDao,
};
#[derive(Clone, Debug)]
pub struct BountyContestSrv {
    bounty_contest_dao: BountyContestDao,
}
use crate::serialize::bounty_contest::BountyContestLeaderboardRes;
use crate::serialize::error::AppError;
use crate::serialize::PaginationRes;

impl BountyContestSrv {
    pub fn new(bounty_contest_dao: BountyContestDao) -> Self {
        BountyContestSrv {
            bounty_contest_dao: bounty_contest_dao.clone(),
        }
    }

    pub async fn process_points(
        &self,
        req: Vec<BountyContestLeaderboard>,
    ) -> Result<bool, AppError> {
        for record in &req {
            let row = self
                .bounty_contest_dao
                .get_by_email(&record.email)
                .await
                .unwrap();
            if let Some(row) = row {
                let current_points: i32 = row.points;
                let new_points = record.points + current_points;

                let _ = self
                    .bounty_contest_dao
                    .update_bc(&record.email, new_points)
                    .await;
            } else {
                // Email does not exist, insert a new record
                let _ = self
                    .bounty_contest_dao
                    .insert_bc(&record.email, &record.username, record.points)
                    .await;
            }
        } // check email exist or not

        Ok(true)
    }
    pub async fn get_dashboard(
        &self,
        page: i64,
        per_page: i64,
    ) -> Result<BountyContestLeaderboardRes, AppError> {
        let total_items = self.bounty_contest_dao.get_count().await.unwrap_or(0);
        let res = self
            .bounty_contest_dao
            .get_dashboard(page, per_page)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
            .unwrap();

        let total_page = total_items as f64 / per_page as f64;
        Ok(BountyContestLeaderboardRes {
            items: res,
            pagination: PaginationRes {
                page: page as i64,
                limit: per_page as i64,
                total_records: total_items,
                total_page: total_page.ceil() as i64,
            },
        })
    }
}
