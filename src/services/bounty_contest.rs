use crate::repositories::bounty_contest::BountyContestDao;
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
}
