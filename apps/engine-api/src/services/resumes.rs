#[cfg(test)]
use std::sync::{Arc, Mutex};

use crate::db::repositories::{RepositoryError, ResumesRepository};
use crate::domain::resume::model::{ResumeVersion, UploadResume};

#[derive(Clone)]
enum ResumesServiceBackend {
    Repository(ResumesRepository),
    #[cfg(test)]
    Stub(Arc<ResumesServiceStub>),
}

#[derive(Clone)]
pub struct ResumesService {
    backend: ResumesServiceBackend,
}

impl ResumesService {
    pub fn new(repository: ResumesRepository) -> Self {
        Self {
            backend: ResumesServiceBackend::Repository(repository),
        }
    }

    pub async fn list(&self) -> Result<Vec<ResumeVersion>, RepositoryError> {
        match &self.backend {
            ResumesServiceBackend::Repository(repository) => repository.list().await,
            #[cfg(test)]
            ResumesServiceBackend::Stub(stub) => stub.list(),
        }
    }

    pub async fn get_active(&self) -> Result<Option<ResumeVersion>, RepositoryError> {
        match &self.backend {
            ResumesServiceBackend::Repository(repository) => repository.get_active().await,
            #[cfg(test)]
            ResumesServiceBackend::Stub(stub) => stub.get_active(),
        }
    }

    pub async fn upload(&self, input: UploadResume) -> Result<ResumeVersion, RepositoryError> {
        match &self.backend {
            ResumesServiceBackend::Repository(repository) => repository.upload(&input).await,
            #[cfg(test)]
            ResumesServiceBackend::Stub(stub) => stub.upload(input),
        }
    }

    pub async fn activate(&self, id: &str) -> Result<Option<ResumeVersion>, RepositoryError> {
        match &self.backend {
            ResumesServiceBackend::Repository(repository) => repository.activate(id).await,
            #[cfg(test)]
            ResumesServiceBackend::Stub(stub) => stub.activate(id),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: ResumesServiceStub) -> Self {
        Self {
            backend: ResumesServiceBackend::Stub(Arc::new(stub)),
        }
    }
}

#[cfg(test)]
#[derive(Default)]
pub struct ResumesServiceStub {
    resumes: Mutex<Vec<ResumeVersion>>,
    database_disabled: bool,
}

#[cfg(test)]
impl ResumesServiceStub {
    fn list(&self) -> Result<Vec<ResumeVersion>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .resumes
            .lock()
            .expect("resumes stub mutex should not be poisoned")
            .clone())
    }

    fn get_active(&self) -> Result<Option<ResumeVersion>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .resumes
            .lock()
            .expect("resumes stub mutex should not be poisoned")
            .iter()
            .find(|resume| resume.is_active)
            .cloned())
    }

    fn upload(&self, input: UploadResume) -> Result<ResumeVersion, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut resumes = self
            .resumes
            .lock()
            .expect("resumes stub mutex should not be poisoned");
        for resume in resumes.iter_mut() {
            resume.is_active = false;
        }

        let created = ResumeVersion {
            id: "resume_test_001".to_string(),
            version: resumes.len() as i32 + 1,
            filename: input.filename,
            raw_text: input.raw_text,
            is_active: true,
            uploaded_at: "2026-04-11T00:00:00+00:00".to_string(),
        };
        resumes.push(created.clone());

        Ok(created)
    }

    fn activate(&self, id: &str) -> Result<Option<ResumeVersion>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut resumes = self
            .resumes
            .lock()
            .expect("resumes stub mutex should not be poisoned");
        let exists = resumes.iter().any(|resume| resume.id == id);
        if !exists {
            return Ok(None);
        }

        for resume in resumes.iter_mut() {
            resume.is_active = resume.id == id;
        }

        Ok(resumes.iter().find(|resume| resume.id == id).cloned())
    }
}
