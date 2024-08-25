use crate::{models::bounty_contest::BountyContestLeaderboard, repositories::bounty_contest::BountyContestDao};
#[derive(Clone, Debug)]
pub struct BountyContestSrv {
    bounty_contest_dao: BountyContestDao,
}
use log::Record;
use crate::serialize::error::AppError;
use crate::serialize::PaginationRes;
use crate::serialize::bounty_contest::BountyContestLeaderboardRes;

impl BountyContestSrv {
    pub fn new(bounty_contest_dao: BountyContestDao) -> Self {
        BountyContestSrv {
                bounty_contest_dao: bounty_contest_dao.clone(),
        }
    }
    pub async fn process_points(&self, req: Vec<BountyContestLeaderboard>) -> Result<bool, AppError> {
        for record in &req {
            let row = self.bounty_contest_dao.get_by_email(&record.email).await.unwrap();
            if let Some(row) = row {
                // Email exists, update the points
                let mut current_points: i32 = row.points;
                let new_points = record.points +current_points;


                let updating = self.bounty_contest_dao.update_bc(&record.email, new_points);
            } else {
                // Email does not exist, insert a new record
                let inserting = self.bounty_contest_dao.insert_bc(&record.email, &record.username,record.points);
            }
        }// check email exist or not
        Ok(true)
    }
    pub async fn get_dashboard(&self,page: i16, per_page: i16)  -> Result<BountyContestLeaderboardRes, AppError> {
        let rows = self.bounty_contest_dao.get_dashboard(page, per_page).await;
        let total_items = self.bounty_contest_dao.get_count().await;
        let total_page:i64 = 0;
     
        // let items: Vec<BountyContestLeaderboard> = rows
        //     .iter()
        //     .map(|row| BountyContestLeaderboard {
        //         email: row.get(0),
        //         username: row.get(1),
        //         points: row.get(2),
        //         created_at: row.get(3),
        //         updated_at : row.get(4)
        //     })
        //     .collect();
        Ok(BountyContestLeaderboardRes {
            items: rows.unwrap(),
            pagination: PaginationRes{
                page: page as i64,
                limit: per_page as i64,
                total_records: total_items.unwrap(),
                total_page: total_page,
            }
        })
    }

}