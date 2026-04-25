use std::collections::BTreeMap;

use crate::domain::feedback::model::JobFeedbackState;
use crate::services::behavior::ProfileBehaviorAggregates;
use crate::services::funnel::ProfileFunnelAggregates;

const MAX_LEARNED_BOOST: i16 = 6;
const MAX_LEARNED_PENALTY: i16 = -6;
const SIGNAL_SATURATION: f32 = 6.0;
const HISTORY_SATURATION: f32 = 8.0;
const MIN_EVIDENCE_STRENGTH: f32 = 0.34;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeterministicScoreBucket {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LearnedRerankFeatures {
    pub source_positive_signal: f32,
    pub source_negative_signal: f32,
    pub role_family_positive_signal: f32,
    pub role_family_negative_signal: f32,
    pub save_history_strength: f32,
    pub bad_fit_history_strength: f32,
    pub application_history_strength: f32,
    pub funnel_quality_hint: f32,
    pub deterministic_score_bucket: Option<DeterministicScoreBucket>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LearnedRerankScore {
    pub score_delta: i16,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedRerankCandidate {
    pub job_id: String,
    pub deterministic_score: u8,
    pub source: Option<String>,
    pub role_family: Option<String>,
    pub feedback: JobFeedbackState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedRerankCandidateEvaluation {
    pub job_id: String,
    pub deterministic_score: u8,
    pub learned_score: u8,
    pub score_delta: i16,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedRerankEvaluation {
    pub deterministic_order: Vec<String>,
    pub learned_order: Vec<String>,
    pub candidates: Vec<LearnedRerankCandidateEvaluation>,
}

#[derive(Clone, Default)]
pub struct LearnedRerankerService;

impl LearnedRerankerService {
    pub fn new() -> Self {
        Self
    }

    pub fn build_features(
        &self,
        deterministic_score: u8,
        source: Option<&str>,
        role_family: Option<&str>,
        behavior: &ProfileBehaviorAggregates,
        funnel: &ProfileFunnelAggregates,
        feedback: &JobFeedbackState,
    ) -> LearnedRerankFeatures {
        let source_signal = source.map(|source| {
            signal_for_key(
                source,
                &behavior.save_count_by_source,
                &behavior.hide_count_by_source,
                &behavior.bad_fit_count_by_source,
                &behavior.application_created_count_by_source,
            )
        });
        let role_family_signal = role_family.map(|role_family| {
            signal_for_key(
                role_family,
                &behavior.save_count_by_role_family,
                &behavior.hide_count_by_role_family,
                &behavior.bad_fit_count_by_role_family,
                &behavior.application_created_count_by_role_family,
            )
        });

        let current_saved = usize::from(feedback.saved);
        let current_bad_fit = usize::from(feedback.bad_fit);
        let source_saves = source_signal.map_or(0, |signal| signal.save_count);
        let role_saves = role_family_signal.map_or(0, |signal| signal.save_count);
        let source_bad_fits = source_signal.map_or(0, |signal| signal.bad_fit_count);
        let role_bad_fits = role_family_signal.map_or(0, |signal| signal.bad_fit_count);
        let source_applications = source_signal.map_or(0, |signal| signal.application_count);
        let role_applications = role_family_signal.map_or(0, |signal| signal.application_count);

        LearnedRerankFeatures {
            source_positive_signal: positive_signal_strength(source_signal),
            source_negative_signal: negative_signal_strength(source_signal),
            role_family_positive_signal: positive_signal_strength(role_family_signal),
            role_family_negative_signal: negative_signal_strength(role_family_signal),
            save_history_strength: strength(
                source_saves + role_saves + current_saved,
                HISTORY_SATURATION,
            ),
            bad_fit_history_strength: strength(
                source_bad_fits + role_bad_fits + current_bad_fit,
                HISTORY_SATURATION,
            ),
            application_history_strength: strength(
                (source_applications + role_applications) * 2,
                HISTORY_SATURATION,
            ),
            funnel_quality_hint: funnel_quality_hint(source, funnel),
            deterministic_score_bucket: Some(deterministic_score_bucket(deterministic_score)),
        }
    }

    pub fn score_features(&self, features: &LearnedRerankFeatures) -> LearnedRerankScore {
        let evidence_strength = features
            .source_positive_signal
            .max(features.source_negative_signal)
            .max(features.role_family_positive_signal)
            .max(features.role_family_negative_signal)
            .max(features.save_history_strength)
            .max(features.bad_fit_history_strength)
            .max(features.application_history_strength)
            .max(features.funnel_quality_hint.abs());

        if evidence_strength < MIN_EVIDENCE_STRENGTH {
            return LearnedRerankScore::default();
        }

        let mut raw_delta = (features.source_positive_signal * 2.0)
            - (features.source_negative_signal * 2.5)
            + (features.role_family_positive_signal * 1.5)
            - (features.role_family_negative_signal * 1.75)
            + (features.save_history_strength * 1.0)
            - (features.bad_fit_history_strength * 1.25)
            + (features.application_history_strength * 1.5)
            + features.funnel_quality_hint;

        match features.deterministic_score_bucket {
            Some(DeterministicScoreBucket::Low) if raw_delta > 0.0 => raw_delta *= 0.5,
            Some(DeterministicScoreBucket::High) if raw_delta < 0.0 => raw_delta *= 0.75,
            _ => {}
        }

        let score_delta = (raw_delta.round() as i16).clamp(MAX_LEARNED_PENALTY, MAX_LEARNED_BOOST);
        if score_delta == 0 {
            return LearnedRerankScore::default();
        }

        LearnedRerankScore {
            score_delta,
            reasons: learned_reasons(features, score_delta),
        }
    }

    pub fn score_job_learned(
        &self,
        deterministic_score: u8,
        source: Option<&str>,
        role_family: Option<&str>,
        behavior: &ProfileBehaviorAggregates,
        funnel: &ProfileFunnelAggregates,
        feedback: &JobFeedbackState,
    ) -> LearnedRerankScore {
        let features = self.build_features(
            deterministic_score,
            source,
            role_family,
            behavior,
            funnel,
            feedback,
        );

        self.score_features(&features)
    }

    pub fn evaluate(
        &self,
        candidates: Vec<LearnedRerankCandidate>,
        behavior: &ProfileBehaviorAggregates,
        funnel: &ProfileFunnelAggregates,
    ) -> LearnedRerankEvaluation {
        let mut deterministic_order = candidates
            .iter()
            .map(|candidate| (candidate.job_id.clone(), candidate.deterministic_score))
            .collect::<Vec<_>>();
        deterministic_order
            .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

        let mut evaluations = candidates
            .into_iter()
            .map(|candidate| {
                let learned_score = self.score_job_learned(
                    candidate.deterministic_score,
                    candidate.source.as_deref(),
                    candidate.role_family.as_deref(),
                    behavior,
                    funnel,
                    &candidate.feedback,
                );
                let final_score = (candidate.deterministic_score as i16 + learned_score.score_delta)
                    .clamp(0, 100) as u8;

                LearnedRerankCandidateEvaluation {
                    job_id: candidate.job_id,
                    deterministic_score: candidate.deterministic_score,
                    learned_score: final_score,
                    score_delta: learned_score.score_delta,
                    reasons: learned_score.reasons,
                }
            })
            .collect::<Vec<_>>();
        evaluations.sort_by(|left, right| {
            right
                .learned_score
                .cmp(&left.learned_score)
                .then_with(|| left.job_id.cmp(&right.job_id))
        });

        LearnedRerankEvaluation {
            deterministic_order: deterministic_order
                .into_iter()
                .map(|(job_id, _)| job_id)
                .collect(),
            learned_order: evaluations
                .iter()
                .map(|candidate| candidate.job_id.clone())
                .collect(),
            candidates: evaluations,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SignalSnapshot {
    save_count: usize,
    hide_count: usize,
    bad_fit_count: usize,
    application_count: usize,
    net_score: i32,
}

fn signal_for_key(
    key: &str,
    saves: &BTreeMap<String, usize>,
    hides: &BTreeMap<String, usize>,
    bad_fits: &BTreeMap<String, usize>,
    applications: &BTreeMap<String, usize>,
) -> SignalSnapshot {
    let save_count = saves.get(key).copied().unwrap_or_default();
    let hide_count = hides.get(key).copied().unwrap_or_default();
    let bad_fit_count = bad_fits.get(key).copied().unwrap_or_default();
    let application_count = applications.get(key).copied().unwrap_or_default();

    SignalSnapshot {
        save_count,
        hide_count,
        bad_fit_count,
        application_count,
        net_score: save_count as i32 + (application_count as i32 * 2)
            - hide_count as i32
            - bad_fit_count as i32,
    }
}

fn positive_signal_strength(signal: Option<SignalSnapshot>) -> f32 {
    signal
        .map(|signal| strength_i32(signal.net_score.max(0), SIGNAL_SATURATION))
        .unwrap_or_default()
}

fn negative_signal_strength(signal: Option<SignalSnapshot>) -> f32 {
    signal
        .map(|signal| strength_i32((-signal.net_score).max(0), SIGNAL_SATURATION))
        .unwrap_or_default()
}

fn strength(count: usize, saturation: f32) -> f32 {
    ((count as f32) / saturation).clamp(0.0, 1.0)
}

fn strength_i32(count: i32, saturation: f32) -> f32 {
    ((count as f32) / saturation).clamp(0.0, 1.0)
}

fn funnel_quality_hint(source: Option<&str>, funnel: &ProfileFunnelAggregates) -> f32 {
    let Some(source) = source else {
        return 0.0;
    };

    let source_impressions = funnel
        .impression_count_by_source
        .get(source)
        .copied()
        .unwrap_or_default();
    if source_impressions < 4 || funnel.impression_count < 4 {
        return 0.0;
    }

    let source_applications = funnel
        .application_created_count_by_source
        .get(source)
        .copied()
        .unwrap_or_default();
    let source_rate = source_applications as f32 / source_impressions as f32;
    let global_rate = funnel.application_created_count as f32 / funnel.impression_count as f32;

    if source_rate >= global_rate + 0.10 && source_applications > 0 {
        0.75
    } else if source_rate + 0.10 <= global_rate && source_impressions >= 6 {
        -0.5
    } else {
        0.0
    }
}

fn deterministic_score_bucket(score: u8) -> DeterministicScoreBucket {
    match score {
        0..=39 => DeterministicScoreBucket::Low,
        40..=74 => DeterministicScoreBucket::Medium,
        _ => DeterministicScoreBucket::High,
    }
}

fn learned_reasons(features: &LearnedRerankFeatures, score_delta: i16) -> Vec<String> {
    let mut reasons = Vec::new();

    if features.source_positive_signal >= MIN_EVIDENCE_STRENGTH && score_delta > 0 {
        reasons.push("Learned reranker boosted this source based on positive outcomes".to_string());
    }
    if features.source_negative_signal >= MIN_EVIDENCE_STRENGTH && score_delta < 0 {
        reasons.push(
            "Learned reranker penalized this source due to repeated negative outcomes".to_string(),
        );
    }
    if features.role_family_positive_signal >= MIN_EVIDENCE_STRENGTH && score_delta > 0 {
        reasons.push(
            "Learned reranker boosted this role family based on positive outcomes".to_string(),
        );
    }
    if features.role_family_negative_signal >= MIN_EVIDENCE_STRENGTH && score_delta < 0 {
        reasons.push(
            "Learned reranker penalized this role family due to repeated negative outcomes"
                .to_string(),
        );
    }
    if features.application_history_strength >= MIN_EVIDENCE_STRENGTH && score_delta > 0 {
        reasons.push(
            "Past applications suggest this source converts better for this profile".to_string(),
        );
    }
    if features.funnel_quality_hint > 0.0 && score_delta > 0 {
        reasons.push(
            "Funnel history suggests this source converts better for this profile".to_string(),
        );
    }
    if features.funnel_quality_hint < 0.0 && score_delta < 0 {
        reasons.push(
            "Funnel history suggests this source converts worse for this profile".to_string(),
        );
    }

    if reasons.is_empty() {
        if score_delta > 0 {
            reasons.push("Learned reranker applied a conservative positive adjustment".to_string());
        } else {
            reasons.push("Learned reranker applied a conservative negative adjustment".to_string());
        }
    }

    reasons
}

#[cfg(test)]
mod tests {
    use crate::domain::feedback::model::JobFeedbackState;
    use crate::services::behavior::ProfileBehaviorAggregates;
    use crate::services::funnel::ProfileFunnelAggregates;

    use super::{
        LearnedRerankCandidate, LearnedRerankerService, MAX_LEARNED_BOOST, MAX_LEARNED_PENALTY,
    };

    #[test]
    fn positive_source_history_creates_bounded_boost() {
        let service = LearnedRerankerService::new();
        let mut behavior = ProfileBehaviorAggregates::default();
        behavior
            .save_count_by_source
            .insert("djinni".to_string(), 5);
        behavior
            .application_created_count_by_source
            .insert("djinni".to_string(), 1);

        let score = service.score_job_learned(
            72,
            Some("djinni"),
            None,
            &behavior,
            &ProfileFunnelAggregates::default(),
            &JobFeedbackState::default(),
        );

        assert!(score.score_delta > 0);
        assert!(score.score_delta <= MAX_LEARNED_BOOST);
        assert!(
            score
                .reasons
                .iter()
                .any(|reason| reason.contains("boosted this source"))
        );
    }

    #[test]
    fn negative_source_history_creates_bounded_penalty() {
        let service = LearnedRerankerService::new();
        let mut behavior = ProfileBehaviorAggregates::default();
        behavior
            .hide_count_by_source
            .insert("work_ua".to_string(), 4);
        behavior
            .bad_fit_count_by_source
            .insert("work_ua".to_string(), 2);

        let score = service.score_job_learned(
            72,
            Some("work_ua"),
            None,
            &behavior,
            &ProfileFunnelAggregates::default(),
            &JobFeedbackState::default(),
        );

        assert!(score.score_delta < 0);
        assert!(score.score_delta >= MAX_LEARNED_PENALTY);
        assert!(
            score
                .reasons
                .iter()
                .any(|reason| reason.contains("penalized this source"))
        );
    }

    #[test]
    fn positive_role_family_history_creates_bounded_boost() {
        let service = LearnedRerankerService::new();
        let mut behavior = ProfileBehaviorAggregates::default();
        behavior
            .save_count_by_role_family
            .insert("product".to_string(), 4);
        behavior
            .application_created_count_by_role_family
            .insert("product".to_string(), 1);

        let score = service.score_job_learned(
            68,
            None,
            Some("product"),
            &behavior,
            &ProfileFunnelAggregates::default(),
            &JobFeedbackState::default(),
        );

        assert!(score.score_delta > 0);
        assert!(score.score_delta <= MAX_LEARNED_BOOST);
        assert!(
            score
                .reasons
                .iter()
                .any(|reason| reason.contains("boosted this role family"))
        );
    }

    #[test]
    fn negative_role_family_history_creates_bounded_penalty() {
        let service = LearnedRerankerService::new();
        let mut behavior = ProfileBehaviorAggregates::default();
        behavior
            .hide_count_by_role_family
            .insert("sales".to_string(), 3);
        behavior
            .bad_fit_count_by_role_family
            .insert("sales".to_string(), 3);

        let score = service.score_job_learned(
            68,
            None,
            Some("sales"),
            &behavior,
            &ProfileFunnelAggregates::default(),
            &JobFeedbackState::default(),
        );

        assert!(score.score_delta < 0);
        assert!(score.score_delta >= MAX_LEARNED_PENALTY);
        assert!(
            score
                .reasons
                .iter()
                .any(|reason| reason.contains("penalized this role family"))
        );
    }

    #[test]
    fn deterministic_score_still_dominates_when_learned_evidence_is_weak() {
        let service = LearnedRerankerService::new();
        let mut behavior = ProfileBehaviorAggregates::default();
        behavior
            .save_count_by_source
            .insert("djinni".to_string(), 1);

        let evaluation = service.evaluate(
            vec![
                LearnedRerankCandidate {
                    job_id: "strong-deterministic".to_string(),
                    deterministic_score: 76,
                    source: Some("work_ua".to_string()),
                    role_family: Some("engineering".to_string()),
                    feedback: JobFeedbackState::default(),
                },
                LearnedRerankCandidate {
                    job_id: "weak-learned".to_string(),
                    deterministic_score: 51,
                    source: Some("djinni".to_string()),
                    role_family: Some("engineering".to_string()),
                    feedback: JobFeedbackState::default(),
                },
            ],
            &behavior,
            &ProfileFunnelAggregates::default(),
        );

        assert_eq!(evaluation.deterministic_order[0], "strong-deterministic");
        assert_eq!(evaluation.learned_order[0], "strong-deterministic");
        assert!(
            evaluation
                .candidates
                .iter()
                .all(|candidate| candidate.score_delta == 0)
        );
    }

    #[test]
    fn evaluation_helper_compares_deterministic_and_learned_ordering() {
        let service = LearnedRerankerService::new();
        let mut behavior = ProfileBehaviorAggregates::default();
        behavior
            .application_created_count_by_source
            .insert("djinni".to_string(), 2);
        behavior
            .save_count_by_source
            .insert("djinni".to_string(), 4);

        let evaluation = service.evaluate(
            vec![
                LearnedRerankCandidate {
                    job_id: "job-1".to_string(),
                    deterministic_score: 70,
                    source: Some("work_ua".to_string()),
                    role_family: Some("engineering".to_string()),
                    feedback: JobFeedbackState::default(),
                },
                LearnedRerankCandidate {
                    job_id: "job-2".to_string(),
                    deterministic_score: 69,
                    source: Some("djinni".to_string()),
                    role_family: Some("engineering".to_string()),
                    feedback: JobFeedbackState::default(),
                },
            ],
            &behavior,
            &ProfileFunnelAggregates::default(),
        );

        assert_eq!(evaluation.deterministic_order, vec!["job-1", "job-2"]);
        assert_eq!(evaluation.learned_order, vec!["job-2", "job-1"]);
    }
}
