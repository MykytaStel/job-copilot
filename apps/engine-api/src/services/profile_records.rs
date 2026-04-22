//! Explicit persisted-profile CRUD boundary.
//!
//! Prefer this module when the caller needs stored profile reads/writes.
//! Profile interpretation and search-profile derivation live elsewhere.

pub use crate::services::profiles::ProfilesService as ProfileRecordsService;

#[cfg(test)]
#[allow(unused_imports)]
pub use crate::services::profiles::ProfilesServiceStub as ProfileRecordsServiceStub;
