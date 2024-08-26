use crate::models::bounty_contest::BountyContestLeaderboard;
use serde::{Deserialize, Serialize};

use super::PaginationRes;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BountyContestLeaderboardRes {
    pub items: Vec<BountyContestLeaderboard>,
    pub pagination: PaginationRes,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BountyContestDataReq {
    pub secret: String,
}
