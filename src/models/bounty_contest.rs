use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "bounty_contest_leaderboard")]
pub struct BountyContestLeaderboard {
    pub email: String,
    pub username: String,
    pub points: i32,

    #[serde(skip_serializing)]
    pub created_at: Option<NaiveDateTime>,

    #[serde(skip_serializing)]
    pub updated_at: Option<NaiveDateTime>,
}
