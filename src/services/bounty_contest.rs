use crate::models::address_book::AddressBook;
use crate::models::bounty_contest::BountyContest;
use crate::repositories::bounty_contest::BountyContestDao;
use crate::serialize::error::AppError;
use crate::serialize::PaginationReq;

#[derive(Clone, Debug)]
pub struct BountyContestSrv {
    bounty_contest_dao: BountyContestDao,
}

impl BountyContestSrv {
    pub fn new(bounty_contest_dao: BountyContestDao) -> Self {
        BountyContestSrv {
            bounty_contest_dao: bounty_contest_dao.clone(),
        }
    }

    // TODO: @Broustail : 4
    // Define logic here, from request you can code logic
    // Call to DAO to connect DB it you need

    pub async fn get_dashboard(
        &self,
        pagination: PaginationReq,
    ) -> Result<Vec<BountyContest>, AppError> {
        // TODO: @Broustail : Example call dao to get dashboard data
        self.bounty_contest_dao
            .get_dashboard(pagination.page, pagination.limit)
            .await
            .map_err(|err| AppError::new(500).message(&err.to_string()))
    }
}
