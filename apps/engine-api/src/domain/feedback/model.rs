#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JobFeedbackFlags {
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobFeedbackRecord {
    pub profile_id: String,
    pub job_id: String,
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanyFeedbackStatus {
    Whitelist,
    Blacklist,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompanyFeedbackRecord {
    pub profile_id: String,
    pub company_name: String,
    pub normalized_company_name: String,
    pub status: CompanyFeedbackStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JobFeedbackState {
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub company_status: Option<CompanyFeedbackStatus>,
}

impl JobFeedbackState {
    pub fn from_sources(
        job_feedback: Option<&JobFeedbackRecord>,
        company_feedback: Option<&CompanyFeedbackRecord>,
    ) -> Self {
        Self {
            saved: job_feedback.is_some_and(|record| record.saved),
            hidden: job_feedback.is_some_and(|record| record.hidden),
            bad_fit: job_feedback.is_some_and(|record| record.bad_fit),
            company_status: company_feedback.map(|record| record.status),
        }
    }
}

impl CompanyFeedbackStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Whitelist => "whitelist",
            Self::Blacklist => "blacklist",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "whitelist" => Some(Self::Whitelist),
            "blacklist" => Some(Self::Blacklist),
            _ => None,
        }
    }
}
