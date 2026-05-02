//! Deterministic search matching over a structured `SearchProfile`.
//!
//! Prefer importing this logic through `crate::services::search_ranking`.
//! It is separate from `crate::services::fit_scoring`, which computes a
//! deterministic fit score for a single profile/job pair used by
//! detail/explanation flows.

#[path = "matching/filters.rs"]
mod filters;
#[path = "matching/quality.rs"]
mod quality;
#[path = "matching/roles.rs"]
mod roles;
#[path = "matching/scoring.rs"]
mod scoring;
#[path = "matching/text.rs"]
mod text;

use crate::domain::job::age::assess_job_age;
use crate::domain::job::model::{Job, JobView};
use crate::domain::job::presentation::assess_description_quality;
use crate::domain::matching::{JobFit, JobScoreBreakdown, JobScorePenalty};
use crate::domain::role::RoleId;
use crate::domain::role::catalog::ROLE_CATALOG;
use crate::domain::search::profile::{SearchProfile, SearchRoleCandidate, TargetRegion, WorkMode};
use crate::domain::source::SourceId;
use crate::services::profile_analysis::text::{
    PreparedText, normalize_term_for_output, normalize_text,
};
use crate::services::salary::score_search_salary;

use filters::{
    compute_region_match, compute_seniority_alignment, compute_source_match,
    compute_work_mode_match, is_job_allowed_by_source, job_source,
};
pub(crate) use quality::summarize_match_quality;
use roles::{
    analyze_role_alignment, collect_role_terms, collect_target_roles, infer_role_family_for_job,
};
use scoring::{
    BuildReasonsInput, build_reasons, confidence_factor, is_low_signal_term, penalty_entry,
    push_ignored_term, push_unique_region, push_unique_role, push_unique_string,
    weighted_overlap_ratio,
};
use text::{
    build_searchable_text, build_searchable_text_parts, collect_missing_signal_details,
    collect_missing_signals, evaluate_terms, evaluate_terms_section_aware,
    extract_significant_tokens, merge_terms, parse_skill_sections,
};

const PRIMARY_ROLE_WEIGHT: f32 = 22.0;
const TARGET_ROLE_WEIGHT: f32 = 12.0;
const ROLE_CANDIDATE_WEIGHT: f32 = 10.0;
const PROFILE_SKILL_WEIGHT: f32 = 20.0;
const PROFILE_KEYWORD_WEIGHT: f32 = 8.0;
const SEARCH_TERM_WEIGHT: f32 = 8.0;
const SOURCE_WEIGHT: f32 = 4.0;
const WORK_MODE_WEIGHT: f32 = 5.0;
const REGION_WEIGHT: f32 = 5.0;
const SENIORITY_WEIGHT: f32 = 6.0;
const EXCLUDE_PENALTY_WEIGHT: f32 = 18.0;
const ROLE_MISMATCH_PENALTY_WEIGHT: f32 = 18.0;
const WORK_MODE_MISMATCH_PENALTY_WEIGHT: f32 = 10.0;
const REGION_MISMATCH_PENALTY_WEIGHT: f32 = 8.0;
const SENIORITY_MISMATCH_PENALTY_WEIGHT: f32 = 8.0;
const PARTIAL_ROLE_MATCH_THRESHOLD: f32 = 0.30;
const LOW_SIGNAL_TERM_MATCH_WEIGHT: f32 = 0.80;
const PARTIAL_PHRASE_MATCH_WEIGHT: f32 = 0.90;
const LOW_SIGNAL_TERMS: &[&str] = &[
    "developer",
    "engineer",
    "specialist",
    "manager",
    "experience",
    "experienced",
    "role",
    "roles",
    "work",
    "working",
    "team",
    "teams",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankedJob {
    pub job: JobView,
    pub fit: JobFit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRunResult {
    pub ranked_jobs: Vec<RankedJob>,
    pub total_candidates: usize,
    pub filtered_out_by_source: usize,
    pub filtered_out_hidden: usize,
    pub filtered_out_company_blacklist: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MatchQualitySummary {
    pub low_evidence_jobs: usize,
    pub weak_description_jobs: usize,
    pub role_mismatch_jobs: usize,
    pub seniority_mismatch_jobs: usize,
    pub source_mismatch_jobs: usize,
    pub top_missing_signals: Vec<String>,
}

#[derive(Clone, Default)]
pub struct SearchMatchingService;

#[derive(Clone, Debug)]
struct RoleAlignment {
    matched_roles: Vec<RoleId>,
    job_roles: Vec<RoleId>,
    primary_overlap: f32,
    best_target_overlap: f32,
    best_partial_match: Option<(RoleId, RoleId, f32)>,
    candidate_overlap: f32,
    mismatch_penalty: f32,
}

#[derive(Clone, Debug)]
struct SeniorityAlignment {
    normalized_profile: Option<String>,
    normalized_job: Option<String>,
    score: f32,
    penalty: f32,
}

#[derive(Clone, Debug)]
struct TermSpec {
    normalized: String,
    output: String,
    canonical_normalized: String,
    canonical_output: String,
    significant_tokens: Vec<String>,
}

#[derive(Clone, Debug)]
struct EvaluatedTerms {
    matched_terms: Vec<String>,
    missing_terms: Vec<String>,
    matched_strength: f32,
    eligible_terms: usize,
    total_weight: f32,
}

#[derive(Clone, Debug, Default)]
struct DeterministicPenaltyReasons {
    exclude_terms: Option<String>,
    role_mismatch: Option<String>,
    work_mode_mismatch: Option<String>,
    region_mismatch: Option<String>,
    seniority_mismatch: Option<String>,
}

impl SearchMatchingService {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, search_profile: &SearchProfile, jobs: Vec<JobView>) -> SearchRunResult {
        let total_candidates = jobs.len();
        let mut filtered_out_by_source = 0usize;
        let mut ranked_jobs = Vec::new();

        for job in jobs {
            if !is_job_allowed_by_source(search_profile, &job) {
                filtered_out_by_source += 1;
                continue;
            }

            let fit = self.score_job_deterministic(search_profile, &job);
            ranked_jobs.push(RankedJob { job, fit });
        }

        ranked_jobs.sort_by(|left, right| {
            right
                .fit
                .score
                .cmp(&left.fit.score)
                .then_with(|| right.job.job.last_seen_at.cmp(&left.job.job.last_seen_at))
                .then_with(|| left.job.job.id.cmp(&right.job.job.id))
        });

        SearchRunResult {
            ranked_jobs,
            total_candidates,
            filtered_out_by_source,
            filtered_out_hidden: 0,
            filtered_out_company_blacklist: 0,
        }
    }

    pub fn score_job_deterministic(&self, search_profile: &SearchProfile, job: &JobView) -> JobFit {
        let prepared_text = PreparedText::new(&build_searchable_text(job));
        let target_roles = collect_target_roles(search_profile);
        let role_alignment = analyze_role_alignment(search_profile, &prepared_text, &target_roles);
        let role_terms = collect_role_terms(&target_roles);
        let skill_sections = parse_skill_sections(&job.job.description_text);
        let matched_profile_skills = evaluate_terms_section_aware(
            &prepared_text,
            &search_profile.profile_skills,
            &[],
            &skill_sections,
        );
        let matched_profile_keywords = evaluate_terms(
            &prepared_text,
            &search_profile.profile_keywords,
            &role_terms,
        );
        let ignored_search_terms = ignored_search_terms(search_profile, &role_terms);
        let matched_search_terms = evaluate_terms(
            &prepared_text,
            &search_profile.search_terms,
            &ignored_search_terms,
        );
        let matched_keywords = merge_terms(
            &matched_profile_keywords.matched_terms,
            &matched_search_terms.matched_terms,
        );
        let source_match = compute_source_match(search_profile, job);
        let work_mode_match = compute_work_mode_match(search_profile, job);
        let region_match = compute_region_match(search_profile, &prepared_text, job);
        let seniority_alignment = compute_seniority_alignment(search_profile, job, &prepared_text);
        let matched_exclude_terms =
            evaluate_terms(&prepared_text, &search_profile.exclude_terms, &[]);
        let missing_signals = collect_missing_signals(
            &matched_profile_skills,
            &matched_profile_keywords,
            &matched_search_terms,
        );
        let missing_signal_details = collect_missing_signal_details(
            &matched_profile_skills,
            &matched_profile_keywords,
            &matched_search_terms,
            &skill_sections,
        );
        let description_quality = assess_description_quality(&job.job.description_text);
        let scoring_weights = search_profile.scoring_weights.clone().normalized();
        let skill_match_multiplier = scoring_weights.skill_match_multiplier();
        let remote_work_multiplier = scoring_weights.remote_work_multiplier();
        let job_freshness_multiplier = scoring_weights.job_freshness_multiplier();
        let salary_fit_multiplier = scoring_weights.salary_fit_multiplier();
        let primary_role_score = role_alignment.primary_overlap
            * PRIMARY_ROLE_WEIGHT
            * confidence_factor(search_profile.primary_role_confidence);
        let target_role_score = role_alignment.best_target_overlap * TARGET_ROLE_WEIGHT;
        let role_candidate_score = role_alignment.candidate_overlap * ROLE_CANDIDATE_WEIGHT;
        let profile_skill_score = weighted_overlap_ratio(
            matched_profile_skills.matched_strength,
            matched_profile_skills.eligible_terms as f32,
        ) * PROFILE_SKILL_WEIGHT
            * skill_match_multiplier;

        let profile_keyword_score = weighted_overlap_ratio(
            matched_profile_keywords.matched_strength,
            matched_profile_keywords.eligible_terms as f32,
        ) * PROFILE_KEYWORD_WEIGHT
            * skill_match_multiplier;

        let search_term_score = weighted_overlap_ratio(
            matched_search_terms.matched_strength,
            matched_search_terms.eligible_terms as f32,
        ) * SEARCH_TERM_WEIGHT
            * skill_match_multiplier;

        let source_score = if source_match && !search_profile.allowed_sources.is_empty() {
            SOURCE_WEIGHT
        } else {
            0.0
        };
        let work_mode_score = matches!(work_mode_match, Some(true))
            .then_some(WORK_MODE_WEIGHT * remote_work_multiplier)
            .unwrap_or(0.0);
        let region_score = matches!(region_match, Some(true))
            .then_some(REGION_WEIGHT)
            .unwrap_or(0.0);
        let seniority_score = seniority_alignment.score;
        let exclude_penalty = weighted_overlap_ratio(
            matched_exclude_terms.matched_strength,
            matched_exclude_terms.total_weight,
        ) * EXCLUDE_PENALTY_WEIGHT;
        let work_mode_penalty = matches!(work_mode_match, Some(false))
            .then_some(WORK_MODE_MISMATCH_PENALTY_WEIGHT * remote_work_multiplier)
            .unwrap_or(0.0);
        let region_mismatch_penalty = matches!(region_match, Some(false))
            .then_some(REGION_MISMATCH_PENALTY_WEIGHT)
            .unwrap_or(0.0);
        let seniority_penalty = seniority_alignment.penalty;

        let matching_score = primary_role_score
            + target_role_score
            + role_candidate_score
            + profile_skill_score
            + profile_keyword_score
            + search_term_score
            + source_score
            + work_mode_score
            + region_score
            + seniority_score;

        let penalty_reasons = DeterministicPenaltyReasons {
            exclude_terms: (!matched_exclude_terms.matched_terms.is_empty()).then_some(format!(
                "Exclude term penalty applied: {}",
                matched_exclude_terms.matched_terms.join(", ")
            )),
            role_mismatch: (role_alignment.mismatch_penalty > 0.0
                && !role_alignment.job_roles.is_empty())
            .then_some(format!(
                "Role mismatch penalty applied: strongest job roles {}",
                role_alignment
                    .job_roles
                    .iter()
                    .take(3)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            work_mode_mismatch: matches!(work_mode_match, Some(false))
                .then_some("Work mode mismatch penalty applied".to_string()),
            region_mismatch: matches!(region_match, Some(false))
                .then_some("Region mismatch penalty applied".to_string()),
            seniority_mismatch: match (
                seniority_alignment.normalized_profile.as_deref(),
                seniority_alignment.normalized_job.as_deref(),
            ) {
                (Some(profile_seniority), Some(job_seniority))
                    if profile_seniority != job_seniority =>
                {
                    Some(format!(
                        "Seniority mismatch penalty applied: profile {} vs job {}",
                        profile_seniority, job_seniority
                    ))
                }
                _ => None,
            },
        };
        let penalties = vec![
            penalty_entry(
                "exclude_terms",
                exclude_penalty,
                penalty_reasons.exclude_terms.clone(),
            ),
            penalty_entry(
                "role_mismatch",
                role_alignment.mismatch_penalty,
                penalty_reasons.role_mismatch.clone(),
            ),
            penalty_entry(
                "work_mode_mismatch",
                work_mode_penalty,
                penalty_reasons.work_mode_mismatch.clone(),
            ),
            penalty_entry(
                "region_mismatch",
                region_mismatch_penalty,
                penalty_reasons.region_mismatch.clone(),
            ),
            penalty_entry(
                "seniority_mismatch",
                seniority_penalty,
                penalty_reasons.seniority_mismatch.clone(),
            ),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        let age_signal = assess_job_age(job);
        let freshness_score =
            (f32::from(age_signal.score_delta) * job_freshness_multiplier).round() as i16;
        let salary_result =
            score_search_salary(search_profile.salary_expectation.as_ref(), &job.job);
        let salary_score =
            (f32::from(salary_result.score_delta) * salary_fit_multiplier).round() as i16;
        let mut reasons = build_reasons(
            search_profile,
            job,
            BuildReasonsInput {
                role_alignment: &role_alignment,
                matched_profile_skills: &matched_profile_skills.matched_terms,
                matched_profile_keywords: &matched_profile_keywords.matched_terms,
                matched_search_terms: &matched_search_terms.matched_terms,
                matched_exclude_terms: &matched_exclude_terms.matched_terms,
                work_mode_match,
                region_match,
                seniority_alignment: &seniority_alignment,
            },
        );

        if age_signal.score_delta < 0 {
            reasons.push(format!(
                "Job age penalty applied: job is {} days old via {} ({} points)",
                age_signal.days_old, age_signal.source, age_signal.score_delta
            ));
        }
        if salary_result.score_delta != 0 {
            if let Some(reason) = salary_result.reason.clone() {
                reasons.push(reason);
            }
        }
        let score_breakdown = JobScoreBreakdown::new(
            matching_score.round() as i16,
            salary_score,
            freshness_score,
            penalties,
        );
        JobFit {
            job_id: job.job.id.clone(),
            score: score_breakdown.total_score,
            score_breakdown,
            matched_roles: role_alignment.matched_roles,
            matched_skills: matched_profile_skills.matched_terms.clone(),
            matched_keywords,
            source_match,
            work_mode_match,
            region_match,
            missing_signals,
            missing_signal_details,
            description_quality,
            reasons,
        }
    }

    pub fn infer_role_family(&self, job: &JobView) -> Option<String> {
        infer_role_family_for_job(
            &job.job,
            job.primary_variant
                .as_ref()
                .map(|variant| variant.source.as_str()),
        )
        .map(str::to_string)
    }

    pub fn infer_role_family_for_job(&self, job: &Job) -> Option<String> {
        infer_role_family_for_job(job, None).map(str::to_string)
    }
}

fn ignored_search_terms(search_profile: &SearchProfile, role_terms: &[String]) -> Vec<String> {
    let mut ignored = role_terms.to_vec();

    for term in &search_profile.profile_skills {
        push_ignored_term(&mut ignored, term);
    }

    for term in &search_profile.profile_keywords {
        push_ignored_term(&mut ignored, term);
    }

    ignored
}

#[cfg(test)]
#[path = "matching/tests.rs"]
mod tests;
