use std::collections::HashMap;
use std::sync::Mutex;

use crate::db::repositories::{AuthCredential, RepositoryError};

#[derive(Default)]
pub struct AuthCredentialsServiceStub {
    credentials_by_email: Mutex<HashMap<String, AuthCredential>>,
    database_disabled: bool,
}

impl AuthCredentialsServiceStub {
    pub(crate) fn create(
        &self,
        profile_id: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<AuthCredential, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let credential = AuthCredential {
            profile_id: profile_id.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
        };

        let mut credentials = self
            .credentials_by_email
            .lock()
            .expect("auth credentials stub mutex should not be poisoned");

        if credentials.contains_key(email) {
            return Err(RepositoryError::Conflict {
                message: "auth credentials already exist for this email".to_string(),
            });
        }

        credentials.insert(email.to_string(), credential.clone());

        Ok(credential)
    }

    pub(crate) fn get_by_email(
        &self,
        email: &str,
    ) -> Result<Option<AuthCredential>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .credentials_by_email
            .lock()
            .expect("auth credentials stub mutex should not be poisoned")
            .get(email)
            .cloned())
    }
}
