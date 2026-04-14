use std::collections::{BTreeMap, BTreeSet};

use crate::domain::user_event::model::{UserEventRecord, UserEventType};

const MIN_SIGNAL_SCORE: i32 = 2;
const STRONG_SIGNAL_SCORE: i32 = 4;
const APPLICATION_CREATED_WEIGHT: i32 = 2;
const TOP_SIGNAL_LIMIT: usize = 3;

const SOURCE_POSITIVE_BOOST: i16 = 2;
const SOURCE_STRONG_POSITIVE_BOOST: i16 = 4;
const SOURCE_NEGATIVE_PENALTY: i16 = -2;
const SOURCE_STRONG_NEGATIVE_PENALTY: i16 = -4;
const ROLE_FAMILY_POSITIVE_BOOST: i16 = 1;
const ROLE_FAMILY_STRONG_POSITIVE_BOOST: i16 = 3;
const ROLE_FAMILY_NEGATIVE_PENALTY: i16 = -1;
const ROLE_FAMILY_STRONG_NEGATIVE_PENALTY: i16 = -3;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfileBehaviorAggregates {
    pub save_count_by_source: BTreeMap<String, usize>,
    pub hide_count_by_source: BTreeMap<String, usize>,
    pub bad_fit_count_by_source: BTreeMap<String, usize>,
    pub search_run_count: usize,
    pub save_count_by_role_family: BTreeMap<String, usize>,
    pub hide_count_by_role_family: BTreeMap<String, usize>,
    pub bad_fit_count_by_role_family: BTreeMap<String, usize>,
    pub application_created_count_by_source: BTreeMap<String, usize>,
    pub application_created_count_by_role_family: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BehaviorSignalCount {
    pub key: String,
    pub save_count: usize,
    pub hide_count: usize,
    pub bad_fit_count: usize,
    pub application_created_count: usize,
    pub positive_count: usize,
    pub negative_count: usize,
    pub net_score: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfileBehaviorSummary {
    pub search_run_count: usize,
    pub top_positive_sources: Vec<BehaviorSignalCount>,
    pub top_negative_sources: Vec<BehaviorSignalCount>,
    pub top_positive_role_families: Vec<BehaviorSignalCount>,
    pub top_negative_role_families: Vec<BehaviorSignalCount>,
    pub source_signal_counts: Vec<BehaviorSignalCount>,
    pub role_family_signal_counts: Vec<BehaviorSignalCount>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BehaviorScoreAdjustment {
    pub score_delta: i16,
    pub reasons: Vec<String>,
}

#[derive(Clone, Default)]
pub struct BehaviorService;

impl BehaviorService {
    pub fn new() -> Self {
        Self
    }

    pub fn build_aggregates<'a>(
        &self,
        events: impl IntoIterator<Item = &'a UserEventRecord>,
    ) -> ProfileBehaviorAggregates {
        let mut aggregates = ProfileBehaviorAggregates::default();

        for event in events {
            match event.event_type {
                UserEventType::JobSaved => {
                    increment_optional(
                        &mut aggregates.save_count_by_source,
                        event.source.as_deref(),
                    );
                    increment_optional(
                        &mut aggregates.save_count_by_role_family,
                        event.role_family.as_deref(),
                    );
                }
                UserEventType::JobHidden => {
                    increment_optional(
                        &mut aggregates.hide_count_by_source,
                        event.source.as_deref(),
                    );
                    increment_optional(
                        &mut aggregates.hide_count_by_role_family,
                        event.role_family.as_deref(),
                    );
                }
                UserEventType::JobBadFit => {
                    increment_optional(
                        &mut aggregates.bad_fit_count_by_source,
                        event.source.as_deref(),
                    );
                    increment_optional(
                        &mut aggregates.bad_fit_count_by_role_family,
                        event.role_family.as_deref(),
                    );
                }
                UserEventType::SearchRun => {
                    aggregates.search_run_count += 1;
                }
                UserEventType::ApplicationCreated => {
                    increment_optional(
                        &mut aggregates.application_created_count_by_source,
                        event.source.as_deref(),
                    );
                    increment_optional(
                        &mut aggregates.application_created_count_by_role_family,
                        event.role_family.as_deref(),
                    );
                }
                _ => {}
            }
        }

        aggregates
    }

    pub fn summarize(&self, aggregates: &ProfileBehaviorAggregates) -> ProfileBehaviorSummary {
        let source_signal_counts = build_signal_counts(
            &aggregates.save_count_by_source,
            &aggregates.hide_count_by_source,
            &aggregates.bad_fit_count_by_source,
            &aggregates.application_created_count_by_source,
        );
        let role_family_signal_counts = build_signal_counts(
            &aggregates.save_count_by_role_family,
            &aggregates.hide_count_by_role_family,
            &aggregates.bad_fit_count_by_role_family,
            &aggregates.application_created_count_by_role_family,
        );

        ProfileBehaviorSummary {
            search_run_count: aggregates.search_run_count,
            top_positive_sources: top_positive(&source_signal_counts),
            top_negative_sources: top_negative(&source_signal_counts),
            top_positive_role_families: top_positive(&role_family_signal_counts),
            top_negative_role_families: top_negative(&role_family_signal_counts),
            source_signal_counts,
            role_family_signal_counts,
        }
    }

    pub fn score_job(
        &self,
        aggregates: &ProfileBehaviorAggregates,
        source: Option<&str>,
        role_family: Option<&str>,
    ) -> BehaviorScoreAdjustment {
        let mut adjustment = BehaviorScoreAdjustment::default();

        if let Some(source) = source {
            let signal = signal_count_for_key(
                source,
                &aggregates.save_count_by_source,
                &aggregates.hide_count_by_source,
                &aggregates.bad_fit_count_by_source,
                &aggregates.application_created_count_by_source,
            );
            let delta = source_adjustment(&signal);

            if delta > 0 {
                adjustment.score_delta += delta;
                adjustment
                    .reasons
                    .push("Source has positive interaction history for this profile".to_string());
            } else if delta < 0 {
                adjustment.score_delta += delta;
                adjustment
                    .reasons
                    .push("Source has repeated hide/bad-fit signals for this profile".to_string());
            }
        }

        if let Some(role_family) = role_family {
            let signal = signal_count_for_key(
                role_family,
                &aggregates.save_count_by_role_family,
                &aggregates.hide_count_by_role_family,
                &aggregates.bad_fit_count_by_role_family,
                &aggregates.application_created_count_by_role_family,
            );
            let delta = role_family_adjustment(&signal);

            if delta > 0 {
                adjustment.score_delta += delta;
                adjustment
                    .reasons
                    .push("Role family has positive interaction history".to_string());
            } else if delta < 0 {
                adjustment.score_delta += delta;
                adjustment
                    .reasons
                    .push("Role family has repeated negative interaction history".to_string());
            }
        }

        adjustment
    }
}

fn increment_optional(target: &mut BTreeMap<String, usize>, key: Option<&str>) {
    let Some(key) = key.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };

    *target.entry(key.to_string()).or_default() += 1;
}

fn build_signal_counts(
    saves: &BTreeMap<String, usize>,
    hides: &BTreeMap<String, usize>,
    bad_fits: &BTreeMap<String, usize>,
    applications: &BTreeMap<String, usize>,
) -> Vec<BehaviorSignalCount> {
    let mut keys = BTreeSet::new();

    for key in saves.keys() {
        keys.insert(key.clone());
    }
    for key in hides.keys() {
        keys.insert(key.clone());
    }
    for key in bad_fits.keys() {
        keys.insert(key.clone());
    }
    for key in applications.keys() {
        keys.insert(key.clone());
    }

    let mut counts = keys
        .into_iter()
        .map(|key| signal_count_for_key(&key, saves, hides, bad_fits, applications))
        .collect::<Vec<_>>();

    counts.sort_by(|left, right| {
        right
            .net_score
            .cmp(&left.net_score)
            .then_with(|| right.positive_count.cmp(&left.positive_count))
            .then_with(|| left.negative_count.cmp(&right.negative_count))
            .then_with(|| left.key.cmp(&right.key))
    });

    counts
}

fn signal_count_for_key(
    key: &str,
    saves: &BTreeMap<String, usize>,
    hides: &BTreeMap<String, usize>,
    bad_fits: &BTreeMap<String, usize>,
    applications: &BTreeMap<String, usize>,
) -> BehaviorSignalCount {
    let save_count = saves.get(key).copied().unwrap_or_default();
    let hide_count = hides.get(key).copied().unwrap_or_default();
    let bad_fit_count = bad_fits.get(key).copied().unwrap_or_default();
    let application_created_count = applications.get(key).copied().unwrap_or_default();
    let positive_count = save_count + application_created_count;
    let negative_count = hide_count + bad_fit_count;

    BehaviorSignalCount {
        key: key.to_string(),
        save_count,
        hide_count,
        bad_fit_count,
        application_created_count,
        positive_count,
        negative_count,
        net_score: signal_net_score(
            save_count,
            hide_count,
            bad_fit_count,
            application_created_count,
        ),
    }
}

fn signal_net_score(
    save_count: usize,
    hide_count: usize,
    bad_fit_count: usize,
    application_created_count: usize,
) -> i32 {
    save_count as i32 + (application_created_count as i32 * APPLICATION_CREATED_WEIGHT)
        - hide_count as i32
        - bad_fit_count as i32
}

fn top_positive(signals: &[BehaviorSignalCount]) -> Vec<BehaviorSignalCount> {
    let mut positive = signals
        .iter()
        .filter(|signal| signal.net_score > 0)
        .cloned()
        .collect::<Vec<_>>();

    positive.sort_by(|left, right| {
        right
            .net_score
            .cmp(&left.net_score)
            .then_with(|| right.positive_count.cmp(&left.positive_count))
            .then_with(|| left.key.cmp(&right.key))
    });
    positive.truncate(TOP_SIGNAL_LIMIT);
    positive
}

fn top_negative(signals: &[BehaviorSignalCount]) -> Vec<BehaviorSignalCount> {
    let mut negative = signals
        .iter()
        .filter(|signal| signal.net_score < 0)
        .cloned()
        .collect::<Vec<_>>();

    negative.sort_by(|left, right| {
        left.net_score
            .cmp(&right.net_score)
            .then_with(|| right.negative_count.cmp(&left.negative_count))
            .then_with(|| left.key.cmp(&right.key))
    });
    negative.truncate(TOP_SIGNAL_LIMIT);
    negative
}

fn source_adjustment(signal: &BehaviorSignalCount) -> i16 {
    signal_delta(
        signal.net_score,
        SOURCE_POSITIVE_BOOST,
        SOURCE_STRONG_POSITIVE_BOOST,
        SOURCE_NEGATIVE_PENALTY,
        SOURCE_STRONG_NEGATIVE_PENALTY,
    )
}

fn role_family_adjustment(signal: &BehaviorSignalCount) -> i16 {
    signal_delta(
        signal.net_score,
        ROLE_FAMILY_POSITIVE_BOOST,
        ROLE_FAMILY_STRONG_POSITIVE_BOOST,
        ROLE_FAMILY_NEGATIVE_PENALTY,
        ROLE_FAMILY_STRONG_NEGATIVE_PENALTY,
    )
}

fn signal_delta(
    net_score: i32,
    mild_positive: i16,
    strong_positive: i16,
    mild_negative: i16,
    strong_negative: i16,
) -> i16 {
    if net_score >= STRONG_SIGNAL_SCORE {
        strong_positive
    } else if net_score >= MIN_SIGNAL_SCORE {
        mild_positive
    } else if net_score <= -STRONG_SIGNAL_SCORE {
        strong_negative
    } else if net_score <= -MIN_SIGNAL_SCORE {
        mild_negative
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::user_event::model::{UserEventRecord, UserEventType};

    use super::BehaviorService;

    fn event(
        id: &str,
        event_type: UserEventType,
        source: Option<&str>,
        role_family: Option<&str>,
    ) -> UserEventRecord {
        UserEventRecord {
            id: id.to_string(),
            profile_id: "profile-1".to_string(),
            event_type,
            job_id: Some(format!("job-{id}")),
            company_name: Some("NovaLedger".to_string()),
            source: source.map(str::to_string),
            role_family: role_family.map(str::to_string),
            payload_json: None,
            created_at: "2026-04-15T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn builds_explicit_behavior_aggregates_and_summary() {
        let service = BehaviorService::new();
        let events = vec![
            event(
                "evt-1",
                UserEventType::JobSaved,
                Some("djinni"),
                Some("engineering"),
            ),
            event(
                "evt-2",
                UserEventType::JobSaved,
                Some("djinni"),
                Some("engineering"),
            ),
            event(
                "evt-3",
                UserEventType::JobHidden,
                Some("work_ua"),
                Some("product"),
            ),
            event(
                "evt-4",
                UserEventType::JobBadFit,
                Some("work_ua"),
                Some("product"),
            ),
            event(
                "evt-5",
                UserEventType::ApplicationCreated,
                Some("djinni"),
                Some("engineering"),
            ),
            event("evt-6", UserEventType::SearchRun, None, Some("engineering")),
        ];

        let aggregates = service.build_aggregates(events.iter());
        let summary = service.summarize(&aggregates);

        assert_eq!(aggregates.search_run_count, 1);
        assert_eq!(aggregates.save_count_by_source.get("djinni"), Some(&2));
        assert_eq!(aggregates.hide_count_by_source.get("work_ua"), Some(&1));
        assert_eq!(aggregates.bad_fit_count_by_source.get("work_ua"), Some(&1));
        assert_eq!(
            aggregates
                .application_created_count_by_role_family
                .get("engineering"),
            Some(&1)
        );

        assert_eq!(summary.top_positive_sources[0].key, "djinni");
        assert_eq!(summary.top_negative_sources[0].key, "work_ua");
        assert_eq!(summary.top_positive_role_families[0].key, "engineering");
        assert_eq!(summary.top_negative_role_families[0].key, "product");
    }

    #[test]
    fn weak_or_balanced_signals_do_not_change_score() {
        let service = BehaviorService::new();
        let events = vec![
            event(
                "evt-1",
                UserEventType::JobSaved,
                Some("djinni"),
                Some("engineering"),
            ),
            event(
                "evt-2",
                UserEventType::JobHidden,
                Some("djinni"),
                Some("engineering"),
            ),
        ];
        let aggregates = service.build_aggregates(events.iter());

        let adjustment = service.score_job(&aggregates, Some("djinni"), Some("engineering"));

        assert_eq!(adjustment.score_delta, 0);
        assert!(adjustment.reasons.is_empty());
    }

    #[test]
    fn impressions_do_not_change_behavior_aggregates() {
        let service = BehaviorService::new();
        let events = vec![
            event(
                "evt-1",
                UserEventType::JobImpression,
                Some("djinni"),
                Some("engineering"),
            ),
            event(
                "evt-2",
                UserEventType::JobSaved,
                Some("djinni"),
                Some("engineering"),
            ),
        ];

        let aggregates = service.build_aggregates(events.iter());
        let summary = service.summarize(&aggregates);

        assert_eq!(aggregates.save_count_by_source.get("djinni"), Some(&1));
        assert_eq!(aggregates.hide_count_by_source.get("djinni"), None);
        assert_eq!(summary.top_positive_sources[0].save_count, 1);
        assert_eq!(summary.top_positive_sources[0].positive_count, 1);
    }
}
