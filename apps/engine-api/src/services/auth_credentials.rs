use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, password_hash};

use crate::db::repositories::{AuthCredential, AuthCredentialsRepository, RepositoryError};

#[cfg(test)]
#[path = "auth_credentials/stub.rs"]
mod stub;

#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
pub use stub::AuthCredentialsServiceStub;

#[derive(Debug)]
pub enum AuthCredentialsError {
    Repository(RepositoryError),
    PasswordHash,
}

impl From<RepositoryError> for AuthCredentialsError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

impl From<password_hash::Error> for AuthCredentialsError {
    fn from(_: password_hash::Error) -> Self {
        Self::PasswordHash
    }
}

#[derive(Clone)]
enum AuthCredentialsServiceBackend {
    Repository(AuthCredentialsRepository),
    #[cfg(test)]
    Stub(Arc<AuthCredentialsServiceStub>),
}

#[derive(Clone)]
pub struct AuthCredentialsService {
    backend: AuthCredentialsServiceBackend,
}

impl AuthCredentialsService {
    pub fn new(repository: AuthCredentialsRepository) -> Self {
        Self {
            backend: AuthCredentialsServiceBackend::Repository(repository),
        }
    }

    pub async fn create(
        &self,
        profile_id: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthCredential, AuthCredentialsError> {
        let password_hash = hash_password(password)?;

        match &self.backend {
            AuthCredentialsServiceBackend::Repository(repository) => repository
                .create(profile_id, email, &password_hash)
                .await
                .map_err(Into::into),
            #[cfg(test)]
            AuthCredentialsServiceBackend::Stub(stub) => stub
                .create(profile_id, email, &password_hash)
                .map_err(Into::into),
        }
    }

    pub async fn get_by_email(
        &self,
        email: &str,
    ) -> Result<Option<AuthCredential>, RepositoryError> {
        match &self.backend {
            AuthCredentialsServiceBackend::Repository(repository) => {
                repository.get_by_email(email).await
            }
            #[cfg(test)]
            AuthCredentialsServiceBackend::Stub(stub) => stub.get_by_email(email),
        }
    }

    pub fn verify_password(&self, password: &str, password_hash: &str) -> bool {
        verify_password(password, password_hash)
    }

    #[cfg(test)]
    pub fn for_tests(stub: AuthCredentialsServiceStub) -> Self {
        Self {
            backend: AuthCredentialsServiceBackend::Stub(Arc::new(stub)),
        }
    }
}

fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn password_hash_verifies_original_password_only() {
        let hash =
            super::hash_password("correct horse battery staple").expect("password should hash");

        assert!(super::verify_password(
            "correct horse battery staple",
            &hash
        ));
        assert!(!super::verify_password("wrong-password", &hash));
        assert_ne!(hash, "correct horse battery staple");
    }
}
