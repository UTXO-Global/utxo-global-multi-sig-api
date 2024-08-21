use serde_derive::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PostgresMapper)]
#[pg_mapper(table = "table_names")] // changge your table name
pub struct BountyContest {
    // TODO: @Broustail : 1
    // Map models to your database
}
