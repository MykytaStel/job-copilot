#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JobFeedbackFlags {
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SalaryFeedbackSignal {
    AboveExpectation,
    AtExpectation,
    BelowExpectation,
    NotShown,
}

impl SalaryFeedbackSignal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AboveExpectation => "above_expectation",
            Self::AtExpectation => "at_expectation",
            Self::BelowExpectation => "below_expectation",
            Self::NotShown => "not_shown",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "above_expectation" => Some(Self::AboveExpectation),
            "at_expectation" => Some(Self::AtExpectation),
            "below_expectation" => Some(Self::BelowExpectation),
            "not_shown" => Some(Self::NotShown),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkModeFeedbackSignal {
    MatchesPreference,
    WouldAccept,
    DealBreaker,
}

impl WorkModeFeedbackSignal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MatchesPreference => "matches_preference",
            Self::WouldAccept => "would_accept",
            Self::DealBreaker => "deal_breaker",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "matches_preference" => Some(Self::MatchesPreference),
            "would_accept" => Some(Self::WouldAccept),
            "deal_breaker" => Some(Self::DealBreaker),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegitimacySignal {
    LooksReal,
    Suspicious,
    Spam,
    Duplicate,
}

impl LegitimacySignal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LooksReal => "looks_real",
            Self::Suspicious => "suspicious",
            Self::Spam => "spam",
            Self::Duplicate => "duplicate",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "looks_real" => Some(Self::LooksReal),
            "suspicious" => Some(Self::Suspicious),
            "spam" => Some(Self::Spam),
            "duplicate" => Some(Self::Duplicate),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobFeedbackReason {
    SalaryTooLow,
    NotRemote,
    TooJunior,
    TooSenior,
    BadTechStack,
    SuspiciousPosting,
    AlreadyApplied,
    DuplicatePosting,
    BadCompanyRep,
    WrongCity,
    WrongIndustry,
    VisaSponsorshipRequired,
    InterestingChallenge,
    GreatCompany,
    GoodSalary,
    RemoteOk,
    GoodTechStack,
    FastGrowthCompany,
    NiceTitle,
}

impl JobFeedbackReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SalaryTooLow => "salary_too_low",
            Self::NotRemote => "not_remote",
            Self::TooJunior => "too_junior",
            Self::TooSenior => "too_senior",
            Self::BadTechStack => "bad_tech_stack",
            Self::SuspiciousPosting => "suspicious_posting",
            Self::AlreadyApplied => "already_applied",
            Self::DuplicatePosting => "duplicate_posting",
            Self::BadCompanyRep => "bad_company_rep",
            Self::WrongCity => "wrong_city",
            Self::WrongIndustry => "wrong_industry",
            Self::VisaSponsorshipRequired => "visa_sponsorship_required",
            Self::InterestingChallenge => "interesting_challenge",
            Self::GreatCompany => "great_company",
            Self::GoodSalary => "good_salary",
            Self::RemoteOk => "remote_ok",
            Self::GoodTechStack => "good_tech_stack",
            Self::FastGrowthCompany => "fast_growth_company",
            Self::NiceTitle => "nice_title",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "salary_too_low" => Some(Self::SalaryTooLow),
            "not_remote" => Some(Self::NotRemote),
            "too_junior" => Some(Self::TooJunior),
            "too_senior" => Some(Self::TooSenior),
            "bad_tech_stack" => Some(Self::BadTechStack),
            "suspicious_posting" => Some(Self::SuspiciousPosting),
            "already_applied" => Some(Self::AlreadyApplied),
            "duplicate_posting" => Some(Self::DuplicatePosting),
            "bad_company_rep" => Some(Self::BadCompanyRep),
            "wrong_city" => Some(Self::WrongCity),
            "wrong_industry" => Some(Self::WrongIndustry),
            "visa_sponsorship_required" => Some(Self::VisaSponsorshipRequired),
            "interesting_challenge" => Some(Self::InterestingChallenge),
            "great_company" => Some(Self::GreatCompany),
            "good_salary" => Some(Self::GoodSalary),
            "remote_ok" => Some(Self::RemoteOk),
            "good_tech_stack" => Some(Self::GoodTechStack),
            "fast_growth_company" => Some(Self::FastGrowthCompany),
            "nice_title" => Some(Self::NiceTitle),
            _ => None,
        }
    }

    pub fn is_negative(self) -> bool {
        matches!(
            self,
            Self::SalaryTooLow
                | Self::NotRemote
                | Self::TooJunior
                | Self::TooSenior
                | Self::BadTechStack
                | Self::SuspiciousPosting
                | Self::AlreadyApplied
                | Self::DuplicatePosting
                | Self::BadCompanyRep
                | Self::WrongCity
                | Self::WrongIndustry
                | Self::VisaSponsorshipRequired
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobFeedbackTagRecord {
    pub profile_id: String,
    pub job_id: String,
    pub tag: JobFeedbackReason,
    pub is_negative: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobFeedbackRecord {
    pub profile_id: String,
    pub job_id: String,
    pub saved: bool,
    pub hidden: bool,
    pub bad_fit: bool,
    pub salary_signal: Option<SalaryFeedbackSignal>,
    pub interest_rating: Option<i8>,
    pub work_mode_signal: Option<WorkModeFeedbackSignal>,
    pub legitimacy_signal: Option<LegitimacySignal>,
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
    pub salary_signal: Option<SalaryFeedbackSignal>,
    pub interest_rating: Option<i8>,
    pub work_mode_signal: Option<WorkModeFeedbackSignal>,
    pub legitimacy_signal: Option<LegitimacySignal>,
    pub tags: Vec<JobFeedbackReason>,
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
            salary_signal: job_feedback.and_then(|record| record.salary_signal),
            interest_rating: job_feedback.and_then(|record| record.interest_rating),
            work_mode_signal: job_feedback.and_then(|record| record.work_mode_signal),
            legitimacy_signal: job_feedback.and_then(|record| record.legitimacy_signal),
            tags: Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salary_signal_parse_and_as_str_are_symmetric() {
        let variants = [
            SalaryFeedbackSignal::AboveExpectation,
            SalaryFeedbackSignal::AtExpectation,
            SalaryFeedbackSignal::BelowExpectation,
            SalaryFeedbackSignal::NotShown,
        ];
        for variant in variants {
            let key = variant.as_str();
            assert_eq!(
                SalaryFeedbackSignal::parse(key),
                Some(variant),
                "parse({key}) failed"
            );
        }
        assert!(SalaryFeedbackSignal::parse("unknown").is_none());
        assert!(SalaryFeedbackSignal::parse("").is_none());
    }

    #[test]
    fn work_mode_signal_parse_and_as_str_are_symmetric() {
        let variants = [
            WorkModeFeedbackSignal::MatchesPreference,
            WorkModeFeedbackSignal::WouldAccept,
            WorkModeFeedbackSignal::DealBreaker,
        ];
        for variant in variants {
            let key = variant.as_str();
            assert_eq!(
                WorkModeFeedbackSignal::parse(key),
                Some(variant),
                "parse({key}) failed"
            );
        }
        assert!(WorkModeFeedbackSignal::parse("unknown").is_none());
    }

    #[test]
    fn legitimacy_signal_parse_and_as_str_are_symmetric() {
        let variants = [
            LegitimacySignal::LooksReal,
            LegitimacySignal::Suspicious,
            LegitimacySignal::Spam,
            LegitimacySignal::Duplicate,
        ];
        for variant in variants {
            let key = variant.as_str();
            assert_eq!(
                LegitimacySignal::parse(key),
                Some(variant),
                "parse({key}) failed"
            );
        }
        assert!(LegitimacySignal::parse("unknown").is_none());
    }

    #[test]
    fn company_status_parse_and_as_str_are_symmetric() {
        for variant in [
            CompanyFeedbackStatus::Whitelist,
            CompanyFeedbackStatus::Blacklist,
        ] {
            let key = variant.as_str();
            assert_eq!(
                CompanyFeedbackStatus::parse(key),
                Some(variant),
                "parse({key}) failed"
            );
        }
        assert!(CompanyFeedbackStatus::parse("unknown").is_none());
    }

    #[test]
    fn feedback_reason_parse_and_as_str_are_symmetric() {
        let variants = [
            JobFeedbackReason::SalaryTooLow,
            JobFeedbackReason::NotRemote,
            JobFeedbackReason::TooJunior,
            JobFeedbackReason::TooSenior,
            JobFeedbackReason::BadTechStack,
            JobFeedbackReason::SuspiciousPosting,
            JobFeedbackReason::AlreadyApplied,
            JobFeedbackReason::DuplicatePosting,
            JobFeedbackReason::BadCompanyRep,
            JobFeedbackReason::WrongCity,
            JobFeedbackReason::WrongIndustry,
            JobFeedbackReason::VisaSponsorshipRequired,
            JobFeedbackReason::InterestingChallenge,
            JobFeedbackReason::GreatCompany,
            JobFeedbackReason::GoodSalary,
            JobFeedbackReason::RemoteOk,
            JobFeedbackReason::GoodTechStack,
            JobFeedbackReason::FastGrowthCompany,
            JobFeedbackReason::NiceTitle,
        ];
        for variant in variants {
            let key = variant.as_str();
            assert_eq!(
                JobFeedbackReason::parse(key),
                Some(variant),
                "parse({key}) failed"
            );
        }
        assert!(JobFeedbackReason::parse("unknown").is_none());
    }

    #[test]
    fn negative_reasons_are_correctly_classified() {
        let negatives = [
            JobFeedbackReason::SalaryTooLow,
            JobFeedbackReason::NotRemote,
            JobFeedbackReason::TooJunior,
            JobFeedbackReason::TooSenior,
            JobFeedbackReason::BadTechStack,
            JobFeedbackReason::SuspiciousPosting,
            JobFeedbackReason::AlreadyApplied,
            JobFeedbackReason::DuplicatePosting,
            JobFeedbackReason::BadCompanyRep,
            JobFeedbackReason::WrongCity,
            JobFeedbackReason::WrongIndustry,
            JobFeedbackReason::VisaSponsorshipRequired,
        ];
        for reason in negatives {
            assert!(reason.is_negative(), "{reason:?} should be negative");
        }
    }

    #[test]
    fn positive_reasons_are_not_negative() {
        let positives = [
            JobFeedbackReason::InterestingChallenge,
            JobFeedbackReason::GreatCompany,
            JobFeedbackReason::GoodSalary,
            JobFeedbackReason::RemoteOk,
            JobFeedbackReason::GoodTechStack,
            JobFeedbackReason::FastGrowthCompany,
            JobFeedbackReason::NiceTitle,
        ];
        for reason in positives {
            assert!(!reason.is_negative(), "{reason:?} should not be negative");
        }
    }

    #[test]
    fn feedback_state_from_none_sources_returns_default() {
        let state = JobFeedbackState::from_sources(None, None);
        assert_eq!(state, JobFeedbackState::default());
    }

    #[test]
    fn feedback_state_from_sources_merges_flags_and_signals() {
        let job_record = JobFeedbackRecord {
            profile_id: "p1".to_string(),
            job_id: "j1".to_string(),
            saved: true,
            hidden: false,
            bad_fit: true,
            salary_signal: Some(SalaryFeedbackSignal::BelowExpectation),
            interest_rating: Some(2),
            work_mode_signal: Some(WorkModeFeedbackSignal::DealBreaker),
            legitimacy_signal: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
        };
        let company_record = CompanyFeedbackRecord {
            profile_id: "p1".to_string(),
            company_name: "Acme".to_string(),
            normalized_company_name: "acme".to_string(),
            status: CompanyFeedbackStatus::Blacklist,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
        };

        let state = JobFeedbackState::from_sources(Some(&job_record), Some(&company_record));

        assert!(state.saved);
        assert!(!state.hidden);
        assert!(state.bad_fit);
        assert_eq!(
            state.salary_signal,
            Some(SalaryFeedbackSignal::BelowExpectation)
        );
        assert_eq!(state.interest_rating, Some(2));
        assert_eq!(
            state.work_mode_signal,
            Some(WorkModeFeedbackSignal::DealBreaker)
        );
        assert_eq!(state.company_status, Some(CompanyFeedbackStatus::Blacklist));
        assert!(state.legitimacy_signal.is_none());
    }

    #[test]
    fn feedback_state_company_none_when_no_company_record() {
        let job_record = JobFeedbackRecord {
            profile_id: "p1".to_string(),
            job_id: "j1".to_string(),
            saved: false,
            hidden: false,
            bad_fit: false,
            salary_signal: None,
            interest_rating: None,
            work_mode_signal: None,
            legitimacy_signal: None,
            created_at: "2026-04-14T00:00:00Z".to_string(),
            updated_at: "2026-04-14T00:00:00Z".to_string(),
        };

        let state = JobFeedbackState::from_sources(Some(&job_record), None);

        assert!(state.company_status.is_none());
    }
}
