use crate::{models::bounty_contest::BountyContestLeaderboard, repositories::bounty_contest::BountyContestDao};
#[derive(Clone, Debug)]
pub struct BountyContestSrv {
    bounty_contest_dao: BountyContestDao,
}
use crate::serialize::error::AppError;
use crate::serialize::PaginationRes;

impl BountyContestSrv {
    pub fn new(bounty_contest_dao: BountyContestDao) -> Self {
        BountyContestSrv {
            bounty_contest_dao: bounty_contest_dao.clone(),
        }
    }
    pub async fn process_points(&self, req: Vec<BountyContestLeaderboard>) -> Result<bool, AppError> {
        for record in &req {
            let row = self.bounty_contest_dao.get_by_email(&record.email);
            if let Some(row) = row {
                // Email exists, update the points
                let current_points: i32 = row.get(0);
                record.points += current_points;
                let updating = self.bounty_contest_dao.update_bc(&record.email, record.points);
            } else {
                // Email does not exist, insert a new record
                let inserting = self.bounty_contest_dao.insert_bc(&record.email, &record.username,record.points);
            }
        }// check email exist or not
        Ok(true)
    }
    pub async fn get_dashboard(&self,page: i64, per_page: i64)  -> Result<BountyContestLeaderboardRes, AppError> {
        let rows = self.bounty_contest_dao.get_dashboard(page, per_page);
        let total_items = self.bounty_contest_dao.get_count();
        let total_page:i64 = 0;
        let items: Vec<Record> = rows
            .iter()
            .map(|row| Record {
                Email: row.get(0),
                Username: row.get(1),
                Points: row.get(2),
            })
            .collect();
        Ok(BountyContestLeaderboardRes {
            items: items,
            pagination: PaginationRes{
                page: page,
                limit: per_page,
                total_records: total_items,
                total_page: total_page,
            }
        });
    }

}