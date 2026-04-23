use crate::db::Database;

mod conversions;
mod queries;
mod rows;
#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct ApplicationsRepository {
    database: Database,
}

impl ApplicationsRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}
