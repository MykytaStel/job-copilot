use std::collections::BTreeMap;

use crate::domain::user_event::model::{UserEventRecord, UserEventType};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfileFunnelAggregates {
    pub impression_count: usize,
    pub open_count: usize,
    pub save_count: usize,
    pub hide_count: usize,
    pub bad_fit_count: usize,
    pub application_created_count: usize,
    pub fit_explanation_requested_count: usize,
    pub application_coach_requested_count: usize,
    pub cover_letter_draft_requested_count: usize,
    pub interview_prep_requested_count: usize,
    pub impression_count_by_source: BTreeMap<String, usize>,
    pub open_count_by_source: BTreeMap<String, usize>,
    pub save_count_by_source: BTreeMap<String, usize>,
    pub application_created_count_by_source: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunnelSourceCount {
    pub source: String,
    pub count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FunnelConversionRates {
    pub open_rate_from_impressions: f64,
    pub save_rate_from_opens: f64,
    pub application_rate_from_saves: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProfileFunnelSummary {
    pub impression_count: usize,
    pub open_count: usize,
    pub save_count: usize,
    pub hide_count: usize,
    pub bad_fit_count: usize,
    pub application_created_count: usize,
    pub fit_explanation_requested_count: usize,
    pub application_coach_requested_count: usize,
    pub cover_letter_draft_requested_count: usize,
    pub interview_prep_requested_count: usize,
    pub conversion_rates: FunnelConversionRates,
    pub impressions_by_source: Vec<FunnelSourceCount>,
    pub opens_by_source: Vec<FunnelSourceCount>,
    pub saves_by_source: Vec<FunnelSourceCount>,
    pub applications_by_source: Vec<FunnelSourceCount>,
}

#[derive(Clone, Default)]
pub struct FunnelService;

impl FunnelService {
    pub fn new() -> Self {
        Self
    }

    pub fn build_aggregates<'a>(
        &self,
        events: impl IntoIterator<Item = &'a UserEventRecord>,
    ) -> ProfileFunnelAggregates {
        let mut aggregates = ProfileFunnelAggregates::default();

        for event in events {
            match event.event_type {
                UserEventType::JobImpression => {
                    aggregates.impression_count += 1;
                    increment_optional(
                        &mut aggregates.impression_count_by_source,
                        event.source.as_deref(),
                    );
                }
                UserEventType::JobOpened => {
                    aggregates.open_count += 1;
                    increment_optional(
                        &mut aggregates.open_count_by_source,
                        event.source.as_deref(),
                    );
                }
                UserEventType::JobSaved => {
                    aggregates.save_count += 1;
                    increment_optional(
                        &mut aggregates.save_count_by_source,
                        event.source.as_deref(),
                    );
                }
                UserEventType::JobHidden => {
                    aggregates.hide_count += 1;
                }
                UserEventType::JobBadFit => {
                    aggregates.bad_fit_count += 1;
                }
                UserEventType::ApplicationCreated => {
                    aggregates.application_created_count += 1;
                    increment_optional(
                        &mut aggregates.application_created_count_by_source,
                        event.source.as_deref(),
                    );
                }
                UserEventType::FitExplanationRequested => {
                    aggregates.fit_explanation_requested_count += 1;
                }
                UserEventType::ApplicationCoachRequested => {
                    aggregates.application_coach_requested_count += 1;
                }
                UserEventType::CoverLetterDraftRequested => {
                    aggregates.cover_letter_draft_requested_count += 1;
                }
                UserEventType::InterviewPrepRequested => {
                    aggregates.interview_prep_requested_count += 1;
                }
                _ => {}
            }
        }

        aggregates
    }

    pub fn summarize(&self, aggregates: &ProfileFunnelAggregates) -> ProfileFunnelSummary {
        ProfileFunnelSummary {
            impression_count: aggregates.impression_count,
            open_count: aggregates.open_count,
            save_count: aggregates.save_count,
            hide_count: aggregates.hide_count,
            bad_fit_count: aggregates.bad_fit_count,
            application_created_count: aggregates.application_created_count,
            fit_explanation_requested_count: aggregates.fit_explanation_requested_count,
            application_coach_requested_count: aggregates.application_coach_requested_count,
            cover_letter_draft_requested_count: aggregates.cover_letter_draft_requested_count,
            interview_prep_requested_count: aggregates.interview_prep_requested_count,
            conversion_rates: FunnelConversionRates {
                open_rate_from_impressions: safe_ratio(
                    aggregates.open_count,
                    aggregates.impression_count,
                ),
                save_rate_from_opens: safe_ratio(aggregates.save_count, aggregates.open_count),
                application_rate_from_saves: safe_ratio(
                    aggregates.application_created_count,
                    aggregates.save_count,
                ),
            },
            impressions_by_source: source_counts(&aggregates.impression_count_by_source),
            opens_by_source: source_counts(&aggregates.open_count_by_source),
            saves_by_source: source_counts(&aggregates.save_count_by_source),
            applications_by_source: source_counts(&aggregates.application_created_count_by_source),
        }
    }
}

fn increment_optional(target: &mut BTreeMap<String, usize>, key: Option<&str>) {
    let Some(key) = key.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };

    *target.entry(key.to_string()).or_default() += 1;
}

fn safe_ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn source_counts(counts: &BTreeMap<String, usize>) -> Vec<FunnelSourceCount> {
    let mut entries = counts
        .iter()
        .map(|(source, count)| FunnelSourceCount {
            source: source.clone(),
            count: *count,
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.source.cmp(&right.source))
    });

    entries
}

#[cfg(test)]
mod tests {
    use crate::domain::user_event::model::{UserEventRecord, UserEventType};

    use super::FunnelService;

    fn event(id: &str, event_type: UserEventType, source: Option<&str>) -> UserEventRecord {
        UserEventRecord {
            id: id.to_string(),
            profile_id: "profile-1".to_string(),
            event_type,
            job_id: Some(format!("job-{id}")),
            company_name: Some("NovaLedger".to_string()),
            source: source.map(str::to_string),
            role_family: Some("engineering".to_string()),
            payload_json: None,
            created_at: "2026-04-15T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn builds_funnel_counts_and_conversion_rates() {
        let service = FunnelService::new();
        let events = vec![
            event("evt-1", UserEventType::JobImpression, Some("djinni")),
            event("evt-2", UserEventType::JobImpression, Some("djinni")),
            event("evt-3", UserEventType::JobImpression, Some("work_ua")),
            event("evt-4", UserEventType::JobOpened, Some("djinni")),
            event("evt-5", UserEventType::JobOpened, Some("djinni")),
            event("evt-6", UserEventType::JobSaved, Some("djinni")),
            event("evt-7", UserEventType::JobHidden, Some("work_ua")),
            event("evt-8", UserEventType::JobBadFit, Some("work_ua")),
            event("evt-9", UserEventType::ApplicationCreated, Some("djinni")),
            event(
                "evt-10",
                UserEventType::FitExplanationRequested,
                Some("djinni"),
            ),
            event(
                "evt-11",
                UserEventType::ApplicationCoachRequested,
                Some("djinni"),
            ),
            event(
                "evt-12",
                UserEventType::CoverLetterDraftRequested,
                Some("djinni"),
            ),
            event(
                "evt-13",
                UserEventType::InterviewPrepRequested,
                Some("djinni"),
            ),
        ];

        let aggregates = service.build_aggregates(events.iter());
        let summary = service.summarize(&aggregates);

        assert_eq!(summary.impression_count, 3);
        assert_eq!(summary.open_count, 2);
        assert_eq!(summary.save_count, 1);
        assert_eq!(summary.hide_count, 1);
        assert_eq!(summary.bad_fit_count, 1);
        assert_eq!(summary.application_created_count, 1);
        assert_eq!(summary.fit_explanation_requested_count, 1);
        assert_eq!(summary.application_coach_requested_count, 1);
        assert_eq!(summary.cover_letter_draft_requested_count, 1);
        assert_eq!(summary.interview_prep_requested_count, 1);
        assert!((summary.conversion_rates.open_rate_from_impressions - (2.0 / 3.0)).abs() < 1e-9);
        assert!((summary.conversion_rates.save_rate_from_opens - 0.5).abs() < 1e-9);
        assert!((summary.conversion_rates.application_rate_from_saves - 1.0).abs() < 1e-9);
        assert_eq!(summary.impressions_by_source[0].source, "djinni");
        assert_eq!(summary.impressions_by_source[0].count, 2);
        assert_eq!(summary.opens_by_source[0].source, "djinni");
        assert_eq!(summary.saves_by_source[0].count, 1);
        assert_eq!(summary.applications_by_source[0].count, 1);
    }

    #[test]
    fn conversion_rates_are_zero_when_funnel_denominators_are_empty() {
        let service = FunnelService::new();
        let events = [event("evt-1", UserEventType::JobSaved, Some("djinni"))];

        let aggregates = service.build_aggregates(events.iter());
        let summary = service.summarize(&aggregates);

        assert_eq!(summary.impression_count, 0);
        assert_eq!(summary.open_count, 0);
        assert_eq!(summary.conversion_rates.open_rate_from_impressions, 0.0);
        assert_eq!(summary.conversion_rates.save_rate_from_opens, 0.0);
        assert_eq!(summary.conversion_rates.application_rate_from_saves, 0.0);
    }
}
