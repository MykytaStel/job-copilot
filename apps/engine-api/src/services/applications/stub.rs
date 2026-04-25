use std::collections::HashMap;
use std::sync::Mutex;

use crate::db::repositories::RepositoryError;
use crate::domain::application::model::{
    Application, ApplicationContact, ApplicationDetail, ApplicationNote, Contact,
    CreateApplication, CreateApplicationContact, CreateContact, CreateNote, Offer,
    UpdateApplication, UpsertOffer,
};
use crate::domain::search::global::ApplicationSearchHit;

#[derive(Default)]
pub struct ApplicationsServiceStub {
    applications_by_id: Mutex<HashMap<String, Application>>,
    recent_applications: Vec<Application>,
    search_applications: Vec<ApplicationSearchHit>,
    details_by_id: Mutex<HashMap<String, ApplicationDetail>>,
    contacts_by_id: Mutex<HashMap<String, Contact>>,
    database_disabled: bool,
}

impl ApplicationsServiceStub {
    pub fn with_application(self, application: Application) -> Self {
        self.applications_by_id
            .lock()
            .expect("applications stub mutex should not be poisoned")
            .insert(application.id.clone(), application);
        self
    }

    pub fn with_contact(self, contact: Contact) -> Self {
        self.contacts_by_id
            .lock()
            .expect("contacts stub mutex should not be poisoned")
            .insert(contact.id.clone(), contact);
        self
    }

    pub fn with_search_application(mut self, application: ApplicationSearchHit) -> Self {
        self.search_applications.push(application);
        self
    }

    pub(crate) fn create(
        &self,
        application: CreateApplication,
    ) -> Result<Application, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let created = Application {
            id: "application_test_001".to_string(),
            job_id: application.job_id,
            resume_id: None,
            status: application.status,
            applied_at: application.applied_at,
            due_date: None,
            outcome: None,
            outcome_date: None,
            rejection_stage: None,
            updated_at: "2026-04-11T00:00:00+00:00".to_string(),
        };

        self.applications_by_id
            .lock()
            .expect("applications stub mutex should not be poisoned")
            .insert(created.id.clone(), created.clone());

        Ok(created)
    }

    pub(crate) fn get_by_id(&self, id: &str) -> Result<Option<Application>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .applications_by_id
            .lock()
            .expect("applications stub mutex should not be poisoned")
            .get(id)
            .cloned())
    }

    pub(crate) fn get_detail_by_id(
        &self,
        id: &str,
    ) -> Result<Option<ApplicationDetail>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .details_by_id
            .lock()
            .expect("application details stub mutex should not be poisoned")
            .get(id)
            .cloned())
    }

    pub(crate) fn list_recent(&self, limit: i64) -> Result<Vec<Application>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .recent_applications
            .iter()
            .take(limit as usize)
            .cloned()
            .collect())
    }

    pub(crate) fn search_by_job_title(
        &self,
        _query: &str,
        limit: i64,
    ) -> Result<Vec<ApplicationSearchHit>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .search_applications
            .iter()
            .take(limit as usize)
            .cloned()
            .collect())
    }

    pub(crate) fn update(
        &self,
        id: &str,
        patch: UpdateApplication,
    ) -> Result<Option<Application>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut applications = self
            .applications_by_id
            .lock()
            .expect("applications stub mutex should not be poisoned");
        let Some(application) = applications.get_mut(id) else {
            return Ok(None);
        };

        if let Some(status) = patch.status {
            application.status = status;
        }
        if let Some(due_date) = patch.due_date {
            application.due_date = due_date;
        }
        application.updated_at = "2026-04-11T00:00:01+00:00".to_string();

        Ok(Some(application.clone()))
    }

    pub(crate) fn create_note(&self, note: CreateNote) -> Result<ApplicationNote, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(ApplicationNote {
            id: "note_test_001".to_string(),
            application_id: note.application_id,
            content: note.content,
            created_at: "2026-04-11T00:00:00+00:00".to_string(),
        })
    }

    pub(crate) fn create_contact(
        &self,
        contact: CreateContact,
    ) -> Result<Contact, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let created = Contact {
            id: "contact_test_001".to_string(),
            name: contact.name,
            email: contact.email,
            phone: contact.phone,
            linkedin_url: contact.linkedin_url,
            company: contact.company,
            role: contact.role,
            created_at: "2026-04-11T00:00:00+00:00".to_string(),
        };

        self.contacts_by_id
            .lock()
            .expect("contacts stub mutex should not be poisoned")
            .insert(created.id.clone(), created.clone());

        Ok(created)
    }

    pub(crate) fn get_contact_by_id(&self, id: &str) -> Result<Option<Contact>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        Ok(self
            .contacts_by_id
            .lock()
            .expect("contacts stub mutex should not be poisoned")
            .get(id)
            .cloned())
    }

    pub(crate) fn list_contacts(
        &self,
        _offset: i64,
    ) -> Result<(Vec<Contact>, i64), RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let contacts: Vec<Contact> = self
            .contacts_by_id
            .lock()
            .expect("contacts stub mutex should not be poisoned")
            .values()
            .cloned()
            .collect();
        let total = contacts.len() as i64;
        Ok((contacts, total))
    }

    pub(crate) fn attach_contact(
        &self,
        contact: CreateApplicationContact,
    ) -> Result<ApplicationContact, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let Some(existing_contact) = self
            .contacts_by_id
            .lock()
            .expect("contacts stub mutex should not be poisoned")
            .get(&contact.contact_id)
            .cloned()
        else {
            return Err(RepositoryError::InvalidData {
                message: format!("Contact '{}' was not found", contact.contact_id),
            });
        };

        let attached = ApplicationContact {
            id: "application_contact_test_001".to_string(),
            application_id: contact.application_id.clone(),
            contact: existing_contact,
            relationship: contact.relationship,
        };

        if let Some(detail) = self
            .details_by_id
            .lock()
            .expect("application details stub mutex should not be poisoned")
            .get_mut(&contact.application_id)
        {
            detail.contacts.push(attached.clone());
        }

        Ok(attached)
    }

    pub(crate) fn upsert_offer(&self, offer: UpsertOffer) -> Result<Offer, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let created = Offer {
            id: "offer_test_001".to_string(),
            application_id: offer.application_id.clone(),
            status: offer.status,
            compensation_min: offer.compensation_min,
            compensation_max: offer.compensation_max,
            compensation_currency: offer.compensation_currency,
            starts_at: offer.starts_at,
            notes: offer.notes,
            created_at: "2026-04-11T00:00:00+00:00".to_string(),
            updated_at: "2026-04-11T00:00:01+00:00".to_string(),
        };

        if let Some(detail) = self
            .details_by_id
            .lock()
            .expect("application details stub mutex should not be poisoned")
            .get_mut(&offer.application_id)
        {
            detail.offer = Some(created.clone());
        }

        Ok(created)
    }

    pub(crate) fn attach_resume(
        &self,
        id: &str,
        resume_id: &str,
    ) -> Result<Option<Application>, RepositoryError> {
        if self.database_disabled {
            return Err(RepositoryError::DatabaseDisabled);
        }

        let mut applications = self
            .applications_by_id
            .lock()
            .expect("applications stub mutex should not be poisoned");
        let Some(application) = applications.get_mut(id) else {
            return Ok(None);
        };

        application.resume_id = Some(resume_id.to_string());
        Ok(Some(application.clone()))
    }
}
