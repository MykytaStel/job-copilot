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
            ResumesServiceBackend::Repository(repository) => {
                if let Some(active) = repository.get_active().await?
                    && normalize_resume_text(&active.raw_text)
                        == normalize_resume_text(&input.raw_text)
                {
                    return Ok(active);
                }

                repository.upload(&input).await
            }
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

    pub async fn delete(&self, id: &str) -> Result<bool, RepositoryError> {
        match &self.backend {
            ResumesServiceBackend::Repository(repository) => repository.delete(id).await,
            #[cfg(test)]
            ResumesServiceBackend::Stub(stub) => stub.delete(id),
        }
    }

    #[cfg(test)]
    pub fn for_tests(stub: ResumesServiceStub) -> Self {
        Self {
            backend: ResumesServiceBackend::Stub(Arc::new(stub)),
        }
    }
}

fn normalize_resume_text(value: &str) -> String {
    value.replace("\r\n", "\n").trim().to_string()
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
        if let Some(active) = resumes.iter().find(|resume| {
            resume.is_active
                && normalize_resume_text(&resume.raw_text) == normalize_resume_text(&input.raw_text)
        }) {
            return Ok(active.clone());
        }

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

    fn delete(&self, id: &str) -> Result<bool, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut resumes = self
            .resumes
            .lock()
            .expect("resumes stub mutex should not be poisoned");
        let original_len = resumes.len();
        resumes.retain(|resume| resume.id != id);

        Ok(resumes.len() != original_len)
    }
}

#[cfg(test)]
mod tests {
    use super::{ResumesService, ResumesServiceStub};
    use crate::domain::resume::model::UploadResume;

    #[tokio::test]
    async fn upload_returns_active_resume_when_text_is_unchanged() {
        let service = ResumesService::for_tests(ResumesServiceStub::default());
        let first = service
            .upload(UploadResume {
                filename: "profile.md".to_string(),
                raw_text: "Senior mobile engineer\nReact Native".to_string(),
            })
            .await
            .expect("first upload should create resume");

        let second = service
            .upload(UploadResume {
                filename: "profile-copy.md".to_string(),
                raw_text: " Senior mobile engineer\nReact Native ".to_string(),
            })
            .await
            .expect("same upload should return active resume");

        assert_eq!(second.id, first.id);
        assert_eq!(service.list().await.expect("list").len(), 1);
    }
}
