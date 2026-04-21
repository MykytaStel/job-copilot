#[path = "jobs/market.rs"]
mod market;
#[path = "jobs/queries.rs"]
mod queries;
#[path = "jobs/rows.rs"]
mod rows;

use crate::db::Database;

#[derive(Clone)]
pub struct JobsRepository {
    database: Database,
}

impl JobsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[cfg(test)]
#[path = "jobs/tests.rs"]
mod tests;
