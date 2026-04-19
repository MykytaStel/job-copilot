use std::collections::BTreeMap;

use crate::domain::job::model::{Job, JobView};
use crate::domain::job::presentation::{JobTextQuality, assess_description_quality};
use crate::domain::matching::JobFit;
use crate::domain::role::RoleId;
use crate::domain::role::catalog::ROLE_CATALOG;
use crate::domain::search::profile::{SearchProfile, SearchRoleCandidate, TargetRegion, WorkMode};
use crate::domain::source::SourceId;
use crate::services::profile::matching::{PreparedText, normalize_term_for_output, normalize_text};

const PRIMARY_ROLE_WEIGHT: f32 = 22.0;
const TARGET_ROLE_WEIGHT: f32 = 12.0;
const ROLE_CANDIDATE_WEIGHT: f32 = 10.0;
const PROFILE_SKILL_WEIGHT: f32 = 20.0;
const PROFILE_KEYWORD_WEIGHT: f32 = 8.0;
const SEARCH_TERM_WEIGHT: f32 = 8.0;
const SOURCE_WEIGHT: f32 = 4.0;
const WORK_MODE_WEIGHT: f32 = 6.0;
const REGION_WEIGHT: f32 = 4.0;
const SENIORITY_WEIGHT: f32 = 6.0;
const EXCLUDE_PENALTY_WEIGHT: f32 = 18.0;
const ROLE_MISMATCH_PENALTY_WEIGHT: f32 = 18.0;
const WORK_MODE_MISMATCH_PENALTY_WEIGHT: f32 = 8.0;
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
}

impl SearchMatchingService {
    pub fn new() -> Self {
        Self
    }

    /// Score and rank all jobs against the given search profile.
    ///
    /// Jobs that are not allowed by the source filter are excluded.
    /// The result is sorted by descending score, then by recency, then by job id.
    /// No truncation is applied here — callers must truncate after any
    /// post-ranking adjustments (e.g. feedback scoring).
    pub fn run(&self, search_profile: &SearchProfile, jobs: Vec<JobView>) -> SearchRunResult {
        let total_candidates = jobs.len();
        let mut filtered_out_by_source = 0usize;
        let mut ranked_jobs = Vec::new();

        for job in jobs {
            if !is_job_allowed_by_source(search_profile, &job) {
                filtered_out_by_source += 1;
                continue;
            }

            let fit = self.score_job(search_profile, &job);
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

    pub fn score_job(&self, search_profile: &SearchProfile, job: &JobView) -> JobFit {
        let prepared_text = PreparedText::new(&build_searchable_text(job));
        let target_roles = collect_target_roles(search_profile);
        let role_alignment = analyze_role_alignment(search_profile, &prepared_text, &target_roles);
        let role_terms = collect_role_terms(&target_roles);
        let matched_profile_skills =
            evaluate_terms(&prepared_text, &search_profile.profile_skills, &[]);
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
        let description_quality = assess_description_quality(&job.job.description_text);

        let primary_role_score = role_alignment.primary_overlap
            * PRIMARY_ROLE_WEIGHT
            * confidence_factor(search_profile.primary_role_confidence);
        let target_role_score = role_alignment.best_target_overlap * TARGET_ROLE_WEIGHT;
        let role_candidate_score = role_alignment.candidate_overlap * ROLE_CANDIDATE_WEIGHT;
        let profile_skill_score = weighted_overlap_ratio(
            matched_profile_skills.matched_strength,
            matched_profile_skills.eligible_terms,
        ) * PROFILE_SKILL_WEIGHT;
        let profile_keyword_score = weighted_overlap_ratio(
            matched_profile_keywords.matched_strength,
            matched_profile_keywords.eligible_terms,
        ) * PROFILE_KEYWORD_WEIGHT;
        let search_term_score = weighted_overlap_ratio(
            matched_search_terms.matched_strength,
            matched_search_terms.eligible_terms,
        ) * SEARCH_TERM_WEIGHT;
        let source_score = if source_match && !search_profile.allowed_sources.is_empty() {
            SOURCE_WEIGHT
        } else {
            0.0
        };
        let work_mode_score = matches!(work_mode_match, Some(true))
            .then_some(WORK_MODE_WEIGHT)
            .unwrap_or(0.0);
        let region_score = match region_match {
            Some(true) => REGION_WEIGHT,
            Some(false) => 0.0,
            None => 0.0,
        };
        let seniority_score = seniority_alignment.score;
        let exclude_penalty = weighted_overlap_ratio(
            matched_exclude_terms.matched_strength,
            matched_exclude_terms.eligible_terms,
        ) * EXCLUDE_PENALTY_WEIGHT;
        let work_mode_penalty = matches!(work_mode_match, Some(false))
            .then_some(WORK_MODE_MISMATCH_PENALTY_WEIGHT)
            .unwrap_or(0.0);
        let seniority_penalty = seniority_alignment.penalty;

        let base_score = primary_role_score
            + target_role_score
            + role_candidate_score
            + profile_skill_score
            + profile_keyword_score
            + search_term_score
            + source_score
            + work_mode_score
            + region_score
            + seniority_score
            - exclude_penalty
            - role_alignment.mismatch_penalty
            - work_mode_penalty
            - seniority_penalty;

        let days_old = days_since_last_seen(&job.job.last_seen_at);
        let freshness_decay = compute_freshness_decay(days_old);
        let score = (base_score * freshness_decay).clamp(0.0, 100.0).round() as u8;

        let mut reasons = build_reasons(
            search_profile,
            job,
            &role_alignment,
            &matched_profile_skills.matched_terms,
            &matched_profile_keywords.matched_terms,
            &matched_search_terms.matched_terms,
            &matched_exclude_terms.matched_terms,
            work_mode_match,
            region_match,
            &seniority_alignment,
        );

        if days_old > 14 {
            reasons.push(format!(
                "Freshness decay applied: job is {} days old (factor {:.2})",
                days_old, freshness_decay
            ));
        }

        JobFit {
            job_id: job.job.id.clone(),
            score,
            matched_roles: role_alignment.matched_roles,
            matched_skills: matched_profile_skills.matched_terms.clone(),
            matched_keywords,
            source_match,
            work_mode_match,
            region_match,
            missing_signals,
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

fn build_searchable_text(job: &JobView) -> String {
    build_searchable_text_parts(
        &job.job,
        job.primary_variant
            .as_ref()
            .map(|variant| variant.source.as_str()),
    )
}

fn build_searchable_text_parts(job: &Job, source: Option<&str>) -> String {
    let mut parts = vec![
        job.title.as_str(),
        job.company_name.as_str(),
        job.description_text.as_str(),
    ];

    if let Some(remote_type) = job.remote_type.as_deref() {
        parts.push(remote_type);
    }

    if let Some(source) = source {
        parts.push(source);
    }

    parts.join(" ")
}

fn infer_role_family_for_job(job: &Job, source: Option<&str>) -> Option<&'static str> {
    let prepared_text = PreparedText::new(&build_searchable_text_parts(job, source));
    let job_roles = collect_job_roles(&prepared_text);
    let mut families = BTreeMap::new();

    for role in job_roles {
        let Some(family) = role.family() else {
            continue;
        };

        *families.entry(family).or_insert(0usize) += 1;
    }

    families
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(family, _)| family)
}

fn collect_target_roles(search_profile: &SearchProfile) -> Vec<RoleId> {
    let mut roles = Vec::new();
    push_unique_role(&mut roles, search_profile.primary_role);

    for role in &search_profile.target_roles {
        push_unique_role(&mut roles, *role);
    }

    roles
}

fn role_matches(prepared_text: &PreparedText, role: RoleId) -> bool {
    if prepared_text.matches_signal(&role.search_label()) {
        return true;
    }

    if prepared_text.matches_signal(role.display_name()) {
        return true;
    }

    role.search_aliases()
        .iter()
        .any(|alias| prepared_text.matches_signal(alias))
}

fn collect_role_terms(target_roles: &[RoleId]) -> Vec<String> {
    let mut terms = Vec::new();

    for role in target_roles {
        push_unique_string(&mut terms, normalize_text(&role.search_label()));
        push_unique_string(&mut terms, normalize_text(role.display_name()));

        for alias in role.search_aliases() {
            push_unique_string(&mut terms, normalize_text(alias));
        }
    }

    terms
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

fn evaluate_terms(
    prepared_text: &PreparedText,
    terms: &[String],
    ignored_terms: &[String],
) -> EvaluatedTerms {
    let mut matched_terms = Vec::new();
    let mut missing_terms = Vec::new();
    let mut matched_strength = 0.0;
    let mut eligible_terms = 0usize;

    for term in build_term_specs(terms) {
        if ignored_terms.contains(&term.normalized)
            || ignored_terms.contains(&term.canonical_normalized)
        {
            continue;
        }

        eligible_terms += 1;

        let Some((output, strength)) = match_term(prepared_text, &term) else {
            push_unique_string(&mut missing_terms, term.canonical_output.clone());
            continue;
        };

        matched_strength += strength;
        push_unique_string(&mut matched_terms, output);
    }

    EvaluatedTerms {
        matched_terms,
        missing_terms,
        matched_strength,
        eligible_terms,
    }
}

fn collect_missing_signals(
    matched_profile_skills: &EvaluatedTerms,
    matched_profile_keywords: &EvaluatedTerms,
    matched_search_terms: &EvaluatedTerms,
) -> Vec<String> {
    let mut missing = Vec::new();

    for term in &matched_profile_skills.missing_terms {
        push_unique_string(&mut missing, term.clone());
    }

    for term in &matched_search_terms.missing_terms {
        push_unique_string(&mut missing, term.clone());
    }

    for term in &matched_profile_keywords.missing_terms {
        push_unique_string(&mut missing, term.clone());
    }

    missing.truncate(8);
    missing
}

fn build_term_specs(terms: &[String]) -> Vec<TermSpec> {
    let mut specs = Vec::new();

    for term in terms {
        let normalized = normalize_text(term);
        if normalized.is_empty() {
            continue;
        }

        let significant_tokens = extract_significant_tokens(&normalized);
        if significant_tokens.is_empty() {
            continue;
        }

        let canonical_normalized = significant_tokens.join(" ");
        if specs
            .iter()
            .any(|spec: &TermSpec| spec.canonical_normalized == canonical_normalized)
        {
            continue;
        }

        specs.push(TermSpec {
            output: normalize_term_for_output(term),
            canonical_output: canonical_normalized.replace('_', " "),
            normalized,
            canonical_normalized,
            significant_tokens,
        });
    }

    specs
}

fn extract_significant_tokens(normalized: &str) -> Vec<String> {
    normalized
        .split_whitespace()
        .filter(|token| !is_low_signal_term(token))
        .map(str::to_string)
        .collect()
}

fn match_term(prepared_text: &PreparedText, term: &TermSpec) -> Option<(String, f32)> {
    if prepared_text.matches_signal(&term.normalized) {
        return Some((term.output.clone(), 1.0));
    }

    if term.significant_tokens.len() == 1 {
        let token = &term.significant_tokens[0];
        if prepared_text.matches_signal(token) {
            let output = if term.normalized == term.canonical_normalized {
                term.output.clone()
            } else {
                term.canonical_output.clone()
            };
            let weight = if term.normalized == term.canonical_normalized {
                1.0
            } else {
                LOW_SIGNAL_TERM_MATCH_WEIGHT
            };
            return Some((output, weight));
        }

        return None;
    }

    if term
        .significant_tokens
        .iter()
        .all(|token| prepared_text.matches_signal(token))
    {
        return Some((term.canonical_output.clone(), PARTIAL_PHRASE_MATCH_WEIGHT));
    }

    None
}

fn is_low_signal_term(token: &str) -> bool {
    !token.contains('_') && LOW_SIGNAL_TERMS.iter().any(|value| value == &token)
}

fn merge_terms(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = Vec::new();

    for term in left {
        push_unique_string(&mut merged, term.clone());
    }

    for term in right {
        push_unique_string(&mut merged, term.clone());
    }

    merged
}

fn analyze_role_alignment(
    search_profile: &SearchProfile,
    prepared_text: &PreparedText,
    target_roles: &[RoleId],
) -> RoleAlignment {
    let job_roles = collect_job_roles(prepared_text);
    let matched_roles = target_roles
        .iter()
        .copied()
        .filter(|role| role_matches(prepared_text, *role))
        .collect::<Vec<_>>();
    let primary_overlap = best_role_overlap(search_profile.primary_role, &job_roles);
    let best_partial_match = best_role_pair(target_roles, &job_roles);
    let best_target_overlap = best_partial_match
        .map(|(_, _, overlap)| overlap)
        .unwrap_or(0.0);
    let candidate_overlap =
        weighted_role_candidate_overlap(&search_profile.role_candidates, &job_roles);
    let mismatch_penalty =
        compute_role_mismatch_penalty(target_roles, &job_roles, best_target_overlap);

    RoleAlignment {
        matched_roles,
        job_roles,
        primary_overlap,
        best_target_overlap,
        best_partial_match,
        candidate_overlap,
        mismatch_penalty,
    }
}

fn collect_job_roles(prepared_text: &PreparedText) -> Vec<RoleId> {
    ROLE_CATALOG
        .iter()
        .filter(|metadata| !metadata.is_fallback && role_matches(prepared_text, metadata.id))
        .map(|metadata| metadata.id)
        .collect()
}

fn weighted_role_candidate_overlap(
    role_candidates: &[SearchRoleCandidate],
    job_roles: &[RoleId],
) -> f32 {
    let total_weight = role_candidates
        .iter()
        .map(|candidate| candidate.confidence as f32)
        .sum::<f32>();

    if total_weight <= 0.0 || job_roles.is_empty() {
        return 0.0;
    }

    let weighted_overlap = role_candidates
        .iter()
        .map(|candidate| best_role_overlap(candidate.role, job_roles) * candidate.confidence as f32)
        .sum::<f32>();

    (weighted_overlap / total_weight).min(1.0)
}

fn best_role_overlap(target_role: RoleId, job_roles: &[RoleId]) -> f32 {
    job_roles
        .iter()
        .map(|job_role| role_family_overlap(target_role, *job_role))
        .fold(0.0, f32::max)
}

fn best_role_pair(target_roles: &[RoleId], job_roles: &[RoleId]) -> Option<(RoleId, RoleId, f32)> {
    let mut best_match = None;

    for target_role in target_roles {
        for job_role in job_roles {
            let overlap = role_family_overlap(*target_role, *job_role);

            if best_match
                .as_ref()
                .map(|(_, _, best_overlap)| overlap > *best_overlap)
                .unwrap_or(true)
            {
                best_match = Some((*target_role, *job_role, overlap));
            }
        }
    }

    best_match
}

fn role_family_overlap(left: RoleId, right: RoleId) -> f32 {
    if left == right {
        return 1.0;
    }

    match (left, right) {
        (RoleId::FrontendEngineer, RoleId::FullstackEngineer)
        | (RoleId::FullstackEngineer, RoleId::FrontendEngineer) => 0.70,
        (RoleId::BackendEngineer, RoleId::FullstackEngineer)
        | (RoleId::FullstackEngineer, RoleId::BackendEngineer) => 0.70,
        (RoleId::MobileEngineer, RoleId::FrontendEngineer)
        | (RoleId::FrontendEngineer, RoleId::MobileEngineer) => 0.40,
        (RoleId::MobileEngineer, RoleId::FullstackEngineer)
        | (RoleId::FullstackEngineer, RoleId::MobileEngineer) => 0.35,
        (RoleId::BackendEngineer, RoleId::DevopsEngineer)
        | (RoleId::DevopsEngineer, RoleId::BackendEngineer) => 0.45,
        (RoleId::FullstackEngineer, RoleId::DevopsEngineer)
        | (RoleId::DevopsEngineer, RoleId::FullstackEngineer) => 0.40,
        (RoleId::DataEngineer, RoleId::MlEngineer) | (RoleId::MlEngineer, RoleId::DataEngineer) => {
            0.50
        }
        (RoleId::TechLead, RoleId::EngineeringManager)
        | (RoleId::EngineeringManager, RoleId::TechLead) => 0.55,
        _ if left.is_fallback() || right.is_fallback() => 0.0,
        _ if left.family().is_some() && left.family() == right.family() => 0.15,
        _ => 0.0,
    }
}

fn compute_role_mismatch_penalty(
    target_roles: &[RoleId],
    job_roles: &[RoleId],
    best_target_overlap: f32,
) -> f32 {
    if job_roles.is_empty()
        || target_roles.iter().all(|role| role.is_fallback())
        || best_target_overlap >= PARTIAL_ROLE_MATCH_THRESHOLD
    {
        return 0.0;
    }

    ROLE_MISMATCH_PENALTY_WEIGHT * (1.0 - best_target_overlap)
}

fn compute_source_match(search_profile: &SearchProfile, job: &JobView) -> bool {
    if search_profile.allowed_sources.is_empty() {
        return true;
    }

    let Some(job_source) = job_source(job) else {
        return false;
    };

    search_profile.allowed_sources.contains(&job_source)
}

fn is_job_allowed_by_source(search_profile: &SearchProfile, job: &JobView) -> bool {
    if search_profile.allowed_sources.is_empty() {
        return true;
    }

    compute_source_match(search_profile, job)
}

fn job_source(job: &JobView) -> Option<SourceId> {
    job.primary_variant
        .as_ref()
        .and_then(|variant| SourceId::parse_canonical_key(&variant.source))
}

fn compute_work_mode_match(search_profile: &SearchProfile, job: &JobView) -> Option<bool> {
    if search_profile.work_modes.is_empty() {
        return None;
    }

    let job_mode = normalize_job_work_mode(job.job.remote_type.as_deref())?;
    Some(search_profile.work_modes.contains(&job_mode))
}

fn normalize_job_work_mode(remote_type: Option<&str>) -> Option<WorkMode> {
    let value = normalize_text(remote_type?);

    match value.as_str() {
        "remote" | "full remote" | "fully remote" => Some(WorkMode::Remote),
        "hybrid" => Some(WorkMode::Hybrid),
        "onsite" | "on site" | "office" | "in office" => Some(WorkMode::Onsite),
        _ => None,
    }
}

fn compute_region_match(
    search_profile: &SearchProfile,
    prepared_text: &PreparedText,
    job: &JobView,
) -> Option<bool> {
    if search_profile.target_regions.is_empty() {
        return None;
    }

    let job_regions = detect_job_regions(prepared_text, job);

    if job_regions.is_empty() {
        return None;
    }

    Some(
        search_profile
            .target_regions
            .iter()
            .any(|region| job_regions.contains(region)),
    )
}

fn detect_job_regions(prepared_text: &PreparedText, job: &JobView) -> Vec<TargetRegion> {
    let mut regions = Vec::new();
    let is_remote = matches!(
        normalize_job_work_mode(job.job.remote_type.as_deref()),
        Some(WorkMode::Remote)
    ) || prepared_text.matches_signal("remote");

    if matches_any(
        prepared_text,
        &["ukraine", "ukrainian", "kyiv", "kyiv oblast"],
    ) {
        push_unique_region(&mut regions, TargetRegion::Ua);
    }

    if matches_any(
        prepared_text,
        &["europe", "european union", "eu timezone", "eu only"],
    ) {
        push_unique_region(&mut regions, TargetRegion::Eu);
    }

    if is_remote
        && matches_any(
            prepared_text,
            &[
                "europe",
                "european union",
                "eu remote",
                "eu timezone",
                "eu only",
            ],
        )
    {
        push_unique_region(&mut regions, TargetRegion::EuRemote);
    }

    if matches_any(prepared_text, &["poland", "warsaw", "krakow", "wroclaw"]) {
        push_unique_region(&mut regions, TargetRegion::Poland);
    }

    if matches_any(prepared_text, &["germany", "berlin", "munich", "hamburg"]) {
        push_unique_region(&mut regions, TargetRegion::Germany);
    }

    if matches_any(
        prepared_text,
        &["united kingdom", "uk", "london", "britain"],
    ) {
        push_unique_region(&mut regions, TargetRegion::Uk);
    }

    if matches_any(
        prepared_text,
        &["united states", "usa", "u s", "new york", "california"],
    ) {
        push_unique_region(&mut regions, TargetRegion::Us);
    }

    regions
}

fn matches_any(prepared_text: &PreparedText, signals: &[&str]) -> bool {
    signals
        .iter()
        .any(|signal| prepared_text.matches_signal(signal))
}

fn compute_seniority_alignment(
    search_profile: &SearchProfile,
    job: &JobView,
    prepared_text: &PreparedText,
) -> SeniorityAlignment {
    let normalized_profile = normalize_seniority(Some(search_profile.seniority.as_str()));
    let normalized_job = normalize_seniority(job.job.seniority.as_deref())
        .or_else(|| infer_seniority_from_text(prepared_text));

    let (Some(profile_seniority), Some(job_seniority)) =
        (normalized_profile.clone(), normalized_job.clone())
    else {
        return SeniorityAlignment {
            normalized_profile,
            normalized_job,
            score: 0.0,
            penalty: 0.0,
        };
    };

    let Some(profile_level) = seniority_ordinal(&profile_seniority) else {
        return SeniorityAlignment {
            normalized_profile,
            normalized_job,
            score: 0.0,
            penalty: 0.0,
        };
    };
    let Some(job_level) = seniority_ordinal(&job_seniority) else {
        return SeniorityAlignment {
            normalized_profile,
            normalized_job,
            score: 0.0,
            penalty: 0.0,
        };
    };

    if profile_level == job_level {
        return SeniorityAlignment {
            normalized_profile,
            normalized_job,
            score: SENIORITY_WEIGHT,
            penalty: 0.0,
        };
    }

    let gap = (profile_level - job_level).unsigned_abs() as f32;
    let direction_multiplier = if profile_level > job_level { 1.0 } else { 0.75 };

    SeniorityAlignment {
        normalized_profile,
        normalized_job,
        score: 0.0,
        penalty: (gap / 2.0).min(1.0) * SENIORITY_MISMATCH_PENALTY_WEIGHT * direction_multiplier,
    }
}

fn normalize_seniority(value: Option<&str>) -> Option<String> {
    match normalize_text(value?).as_str() {
        "intern" => Some("intern".to_string()),
        "junior" | "jr" => Some("junior".to_string()),
        "middle" | "mid" | "mid level" | "midlevel" => Some("middle".to_string()),
        "senior" | "sr" => Some("senior".to_string()),
        "lead" => Some("lead".to_string()),
        "staff" => Some("staff".to_string()),
        "principal" | "head" | "director" => Some("principal".to_string()),
        _ => None,
    }
}

fn infer_seniority_from_text(prepared_text: &PreparedText) -> Option<String> {
    if prepared_text.matches_signal("principal") || prepared_text.matches_signal("director") {
        Some("principal".to_string())
    } else if prepared_text.matches_signal("staff") {
        Some("staff".to_string())
    } else if prepared_text.matches_signal("lead") {
        Some("lead".to_string())
    } else if prepared_text.matches_signal("senior") {
        Some("senior".to_string())
    } else if prepared_text.matches_signal("middle")
        || prepared_text.matches_signal("mid-level")
        || prepared_text.matches_signal("mid level")
        || prepared_text.matches_signal("mid")
    {
        Some("middle".to_string())
    } else if prepared_text.matches_signal("junior") {
        Some("junior".to_string())
    } else if prepared_text.matches_signal("intern") {
        Some("intern".to_string())
    } else {
        None
    }
}

fn seniority_ordinal(value: &str) -> Option<i32> {
    match value {
        "intern" => Some(0),
        "junior" => Some(1),
        "middle" => Some(2),
        "senior" => Some(3),
        "lead" => Some(4),
        "staff" => Some(5),
        "principal" => Some(6),
        _ => None,
    }
}

fn build_reasons(
    search_profile: &SearchProfile,
    job: &JobView,
    role_alignment: &RoleAlignment,
    matched_profile_skills: &[String],
    matched_profile_keywords: &[String],
    matched_search_terms: &[String],
    matched_exclude_terms: &[String],
    work_mode_match: Option<bool>,
    region_match: Option<bool>,
    seniority_alignment: &SeniorityAlignment,
) -> Vec<String> {
    let mut reasons = Vec::new();

    if role_alignment.matched_roles.is_empty() {
        if let Some((target_role, job_role, overlap)) = role_alignment.best_partial_match {
            if overlap >= PARTIAL_ROLE_MATCH_THRESHOLD {
                reasons.push(format!(
                    "Role family overlap matched {} with {}",
                    target_role, job_role
                ));
            } else {
                reasons.push(
                    "No target role signals matched the job title or description".to_string(),
                );
            }
        } else {
            reasons.push("No target role signals matched the job title or description".to_string());
        }
    } else {
        reasons.push(format!(
            "Matched target roles: {}",
            role_alignment
                .matched_roles
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if let Some(confidence) = search_profile.primary_role_confidence {
        reasons.push(format!(
            "Primary role confidence carried into ranking: {}% for {}",
            confidence, search_profile.primary_role
        ));
    }

    if !matched_profile_skills.is_empty() {
        reasons.push(format!(
            "Matched profile skills: {}",
            matched_profile_skills.join(", ")
        ));
    }

    if !matched_profile_keywords.is_empty() {
        reasons.push(format!(
            "Matched profile keywords: {}",
            matched_profile_keywords.join(", ")
        ));
    }

    if !matched_search_terms.is_empty() {
        reasons.push(format!(
            "Matched search terms: {}",
            matched_search_terms.join(", ")
        ));
    }

    if search_profile.allowed_sources.is_empty() {
        reasons.push("All sources are allowed".to_string());
    } else if let Some(source) = job_source(job) {
        reasons.push(format!(
            "Source matched allowed sources: {}",
            source.canonical_key()
        ));
    } else {
        reasons.push("Job source was unavailable".to_string());
    }

    if !search_profile.work_modes.is_empty() {
        match work_mode_match {
            Some(true) => reasons.push("Work mode matched the search profile".to_string()),
            Some(false) => reasons.push("Work mode mismatch penalty applied".to_string()),
            None => reasons.push("Work mode could not be inferred from the job".to_string()),
        }
    }

    if !search_profile.target_regions.is_empty() {
        match region_match {
            Some(true) => reasons.push("Target region matched the job text".to_string()),
            Some(false) => reasons.push("Target region did not match the job text".to_string()),
            None => reasons.push("Region could not be inferred from the job".to_string()),
        }
    }

    match (
        seniority_alignment.normalized_profile.as_deref(),
        seniority_alignment.normalized_job.as_deref(),
    ) {
        (Some(profile_seniority), Some(job_seniority)) if profile_seniority == job_seniority => {
            reasons.push(format!("Matched seniority: {}", profile_seniority));
        }
        (Some(profile_seniority), Some(job_seniority)) => reasons.push(format!(
            "Seniority mismatch penalty applied: profile {} vs job {}",
            profile_seniority, job_seniority
        )),
        _ => {}
    }

    if !matched_exclude_terms.is_empty() {
        reasons.push(format!(
            "Exclude term penalty applied: {}",
            matched_exclude_terms.join(", ")
        ));
    }

    if role_alignment.mismatch_penalty > 0.0 && !role_alignment.job_roles.is_empty() {
        reasons.push(format!(
            "Role mismatch penalty applied: strongest job roles {}",
            role_alignment
                .job_roles
                .iter()
                .take(3)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if matched_profile_skills.is_empty()
        && !search_profile.profile_skills.is_empty()
        && role_alignment.matched_roles.is_empty()
        && matched_search_terms.is_empty()
    {
        reasons.push("No strong profile evidence matched the job text".to_string());
    }

    reasons
}

fn confidence_factor(confidence: Option<u8>) -> f32 {
    match confidence {
        Some(value) => (0.45 + (value as f32 / 100.0) * 0.55).min(1.0),
        None => 0.60,
    }
}

fn weighted_overlap_ratio(matched_strength: f32, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        (matched_strength / total as f32).min(1.0)
    }
}

fn push_unique_role(target: &mut Vec<RoleId>, value: RoleId) {
    if !target.contains(&value) {
        target.push(value);
    }
}

fn push_unique_region(target: &mut Vec<TargetRegion>, value: TargetRegion) {
    if !target.contains(&value) {
        target.push(value);
    }
}

fn push_unique_string(target: &mut Vec<String>, value: String) {
    if value.is_empty() || target.iter().any(|existing| existing == &value) {
        return;
    }

    target.push(value);
}

fn push_ignored_term(target: &mut Vec<String>, value: &str) {
    let normalized = normalize_text(value);

    if normalized.is_empty() {
        return;
    }

    push_unique_string(target, normalized.clone());

    let canonical_tokens = extract_significant_tokens(&normalized);
    if canonical_tokens.is_empty() {
        return;
    }

    push_unique_string(target, canonical_tokens.join(" "));
}

fn days_since_last_seen(last_seen_at: &str) -> i64 {
    let job_day = parse_days_since_epoch(last_seen_at).unwrap_or(0);
    (current_days_since_epoch() - job_day).max(0)
}

fn compute_freshness_decay(days_old: i64) -> f32 {
    if days_old <= 14 {
        return 1.0;
    }
    (1.0 - (days_old as f32 - 14.0) / 30.0).max(0.7)
}

fn parse_days_since_epoch(datetime_str: &str) -> Option<i64> {
    let s = datetime_str.get(..10)?;
    let year: i64 = s[0..4].parse().ok()?;
    let month: i64 = s[5..7].parse().ok()?;
    let day: i64 = s[8..10].parse().ok()?;
    Some(civil_to_days(year, month, day))
}

// Gregorian civil-to-days algorithm (Howard Hinnant, days since 1970-01-01).
fn civil_to_days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn current_days_since_epoch() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        / 86400
}

pub fn summarize_match_quality(ranked_jobs: &[RankedJob]) -> MatchQualitySummary {
    let mut low_evidence_jobs = 0usize;
    let mut weak_description_jobs = 0usize;
    let mut role_mismatch_jobs = 0usize;
    let mut seniority_mismatch_jobs = 0usize;
    let mut source_mismatch_jobs = 0usize;
    let mut missing_counts = BTreeMap::<String, usize>::new();

    for ranked in ranked_jobs {
        let fit = &ranked.fit;

        if fit.matched_roles.is_empty()
            && fit.matched_skills.is_empty()
            && fit.matched_keywords.is_empty()
        {
            low_evidence_jobs += 1;
        }

        if matches!(fit.description_quality, JobTextQuality::Weak) {
            weak_description_jobs += 1;
        }

        if fit
            .reasons
            .iter()
            .any(|reason| reason.contains("Role mismatch penalty applied"))
        {
            role_mismatch_jobs += 1;
        }

        if fit
            .reasons
            .iter()
            .any(|reason| reason.contains("Seniority mismatch penalty applied"))
        {
            seniority_mismatch_jobs += 1;
        }

        if !fit.source_match {
            source_mismatch_jobs += 1;
        }

        for signal in &fit.missing_signals {
            *missing_counts.entry(signal.clone()).or_insert(0usize) += 1;
        }
    }

    let mut top_missing_signals = missing_counts.into_iter().collect::<Vec<_>>();
    top_missing_signals
        .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    MatchQualitySummary {
        low_evidence_jobs,
        weak_description_jobs,
        role_mismatch_jobs,
        seniority_mismatch_jobs,
        source_mismatch_jobs,
        top_missing_signals: top_missing_signals
            .into_iter()
            .take(8)
            .map(|(signal, _)| signal)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
    use crate::domain::role::RoleId;
    use crate::domain::search::profile::{
        SearchProfile, SearchRoleCandidate, TargetRegion, WorkMode,
    };
    use crate::domain::source::SourceId;

    use super::SearchMatchingService;

    fn search_profile() -> SearchProfile {
        SearchProfile {
            primary_role: RoleId::BackendEngineer,
            primary_role_confidence: Some(94),
            target_roles: vec![RoleId::BackendEngineer, RoleId::DevopsEngineer],
            role_candidates: vec![
                SearchRoleCandidate {
                    role: RoleId::BackendEngineer,
                    confidence: 94,
                },
                SearchRoleCandidate {
                    role: RoleId::DevopsEngineer,
                    confidence: 62,
                },
            ],
            seniority: "senior".to_string(),
            target_regions: vec![TargetRegion::EuRemote],
            work_modes: vec![WorkMode::Remote],
            allowed_sources: vec![SourceId::Djinni],
            profile_skills: vec!["rust".to_string(), "postgres".to_string()],
            profile_keywords: vec!["backend".to_string(), "platform".to_string()],
            search_terms: vec![
                "rust".to_string(),
                "postgres".to_string(),
                "distributed systems".to_string(),
            ],
            exclude_terms: vec!["gambling".to_string()],
        }
    }

    fn mobile_profile() -> SearchProfile {
        SearchProfile {
            primary_role: RoleId::MobileEngineer,
            primary_role_confidence: Some(97),
            target_roles: vec![RoleId::MobileEngineer, RoleId::FrontendEngineer],
            role_candidates: vec![
                SearchRoleCandidate {
                    role: RoleId::MobileEngineer,
                    confidence: 97,
                },
                SearchRoleCandidate {
                    role: RoleId::FrontendEngineer,
                    confidence: 68,
                },
            ],
            seniority: "senior".to_string(),
            target_regions: vec![TargetRegion::EuRemote],
            work_modes: vec![WorkMode::Remote],
            allowed_sources: vec![SourceId::Djinni],
            profile_skills: vec!["react native".to_string(), "typescript".to_string()],
            profile_keywords: vec!["mobile".to_string(), "frontend".to_string()],
            search_terms: vec![
                "react native".to_string(),
                "typescript".to_string(),
                "mobile product".to_string(),
            ],
            exclude_terms: vec!["gambling".to_string()],
        }
    }

    fn frontend_profile() -> SearchProfile {
        SearchProfile {
            primary_role: RoleId::FrontendEngineer,
            primary_role_confidence: Some(96),
            target_roles: vec![RoleId::FrontendEngineer, RoleId::MobileEngineer],
            role_candidates: vec![
                SearchRoleCandidate {
                    role: RoleId::FrontendEngineer,
                    confidence: 96,
                },
                SearchRoleCandidate {
                    role: RoleId::MobileEngineer,
                    confidence: 54,
                },
            ],
            seniority: "senior".to_string(),
            target_regions: vec![TargetRegion::EuRemote],
            work_modes: vec![WorkMode::Remote],
            allowed_sources: vec![SourceId::Djinni],
            profile_skills: vec!["react".to_string(), "typescript".to_string()],
            profile_keywords: vec!["frontend".to_string(), "design system".to_string()],
            search_terms: vec![
                "frontend developer".to_string(),
                "react".to_string(),
                "typescript".to_string(),
            ],
            exclude_terms: vec!["gambling".to_string()],
        }
    }

    fn job_view(
        id: &str,
        title: &str,
        description: &str,
        remote_type: Option<&str>,
        source: &str,
    ) -> JobView {
        JobView {
            job: Job {
                id: id.to_string(),
                title: title.to_string(),
                company_name: "NovaLedger".to_string(),
                location: None,
                remote_type: remote_type.map(str::to_string),
                seniority: Some("senior".to_string()),
                description_text: description.to_string(),
                salary_min: None,
                salary_max: None,
                salary_currency: None,
                posted_at: Some("2026-04-10T09:00:00Z".to_string()),
                last_seen_at: "2026-04-14T09:00:00Z".to_string(),
                is_active: true,
            },
            first_seen_at: "2026-04-10T09:00:00Z".to_string(),
            inactivated_at: None,
            reactivated_at: None,
            lifecycle_stage: JobLifecycleStage::Active,
            primary_variant: Some(JobSourceVariant {
                source: source.to_string(),
                source_job_id: format!("{id}-source"),
                source_url: format!("https://example.com/{id}"),
                raw_payload: None,
                fetched_at: "2026-04-14T09:00:00Z".to_string(),
                last_seen_at: "2026-04-14T09:00:00Z".to_string(),
                is_active: true,
                inactivated_at: None,
            }),
        }
    }

    #[test]
    fn matching_role_and_terms_score_higher_than_unrelated_job() {
        let service = SearchMatchingService::new();
        let profile = search_profile();

        let matching_job = job_view(
            "job-1",
            "Senior Backend Developer",
            "Remote EU role working with Rust, Postgres, and distributed systems",
            Some("remote"),
            "djinni",
        );
        let unrelated_job = job_view(
            "job-2",
            "Marketing Specialist",
            "Onsite campaign execution and social media planning",
            Some("onsite"),
            "djinni",
        );

        let matching_fit = service.score_job(&profile, &matching_job);
        let unrelated_fit = service.score_job(&profile, &unrelated_job);

        assert!(matching_fit.score > unrelated_fit.score);
        assert!(
            matching_fit
                .matched_roles
                .contains(&RoleId::BackendEngineer)
        );
        assert!(matching_fit.matched_skills.contains(&"rust".to_string()));
    }

    #[test]
    fn source_mismatch_is_filtered_out_when_allowed_sources_are_set() {
        let service = SearchMatchingService::new();
        let profile = search_profile();

        let results = service.run(
            &profile,
            vec![
                job_view(
                    "job-1",
                    "Senior Backend Developer",
                    "Remote EU role with Rust",
                    Some("remote"),
                    "djinni",
                ),
                job_view(
                    "job-2",
                    "Senior Backend Developer",
                    "Remote EU role with Rust",
                    Some("remote"),
                    "work_ua",
                ),
            ],
        );

        assert_eq!(results.filtered_out_by_source, 1);
        assert_eq!(results.ranked_jobs.len(), 1);
        assert_eq!(results.ranked_jobs[0].job.job.id, "job-1");
    }

    #[test]
    fn empty_allowed_sources_keeps_all_sources_eligible() {
        let service = SearchMatchingService::new();
        let mut profile = search_profile();
        profile.allowed_sources.clear();

        let results = service.run(
            &profile,
            vec![
                job_view(
                    "job-1",
                    "Senior Backend Developer",
                    "Remote EU role with Rust",
                    Some("remote"),
                    "djinni",
                ),
                job_view(
                    "job-2",
                    "Senior Backend Developer",
                    "Remote EU role with Rust",
                    Some("remote"),
                    "work_ua",
                ),
            ],
        );

        assert_eq!(results.filtered_out_by_source, 0);
        assert_eq!(results.ranked_jobs.len(), 2);
        assert!(results.ranked_jobs.iter().all(|job| job.fit.source_match));
    }

    #[test]
    fn search_terms_contribute_to_score() {
        let service = SearchMatchingService::new();
        let profile = search_profile();

        let matching_terms_job = job_view(
            "job-1",
            "Senior Backend Developer",
            "Remote EU role with Rust, Postgres, and distributed systems",
            Some("remote"),
            "djinni",
        );
        let missing_terms_job = job_view(
            "job-2",
            "Senior Backend Developer",
            "Remote EU role with Scala and Cassandra",
            Some("remote"),
            "djinni",
        );

        let matching_terms_fit = service.score_job(&profile, &matching_terms_job);
        let missing_terms_fit = service.score_job(&profile, &missing_terms_job);

        assert!(matching_terms_fit.score > missing_terms_fit.score);
        assert!(
            !matching_terms_fit.matched_keywords.is_empty()
                || !matching_terms_fit.matched_skills.is_empty()
        );
    }

    #[test]
    fn seniority_mismatch_lowers_score() {
        let service = SearchMatchingService::new();
        let profile = search_profile();
        let matching_job = job_view(
            "job-1",
            "Senior Backend Developer",
            "Remote EU role with Rust and Postgres",
            Some("remote"),
            "djinni",
        );
        let junior_job = JobView {
            job: Job {
                seniority: Some("junior".to_string()),
                ..matching_job.job.clone()
            },
            ..matching_job.clone()
        };

        let matching_fit = service.score_job(&profile, &matching_job);
        let junior_fit = service.score_job(&profile, &junior_job);

        assert!(matching_fit.score > junior_fit.score);
        assert!(
            junior_fit
                .reasons
                .iter()
                .any(|reason| reason.contains("Seniority mismatch penalty applied"))
        );
    }

    #[test]
    fn role_family_overlap_gives_partial_credit() {
        let service = SearchMatchingService::new();
        let profile = mobile_profile();
        let exact_job = job_view(
            "job-1",
            "Senior React Native Developer",
            "Remote EU role building React Native apps with TypeScript",
            Some("remote"),
            "djinni",
        );
        let partial_job = job_view(
            "job-2",
            "Senior Fullstack Developer",
            "Remote EU product role with React, TypeScript, and backend APIs",
            Some("remote"),
            "djinni",
        );
        let unrelated_job = job_view(
            "job-3",
            "Senior Backend Developer",
            "Remote EU role with Rust and distributed systems",
            Some("remote"),
            "djinni",
        );

        let exact_fit = service.score_job(&profile, &exact_job);
        let partial_fit = service.score_job(&profile, &partial_job);
        let unrelated_fit = service.score_job(&profile, &unrelated_job);

        assert!(exact_fit.score > partial_fit.score);
        assert!(partial_fit.score > unrelated_fit.score);
        assert!(
            partial_fit
                .reasons
                .iter()
                .any(|reason| reason.contains("Role family overlap"))
        );
    }

    #[test]
    fn exclude_terms_lower_score() {
        let service = SearchMatchingService::new();
        let profile = search_profile();
        let clean_job = job_view(
            "job-1",
            "Senior Backend Developer",
            "Remote EU role with Rust and Postgres for a product platform",
            Some("remote"),
            "djinni",
        );
        let excluded_job = job_view(
            "job-2",
            "Senior Backend Developer",
            "Remote EU role with Rust and Postgres for a gambling platform",
            Some("remote"),
            "djinni",
        );

        let clean_fit = service.score_job(&profile, &clean_job);
        let excluded_fit = service.score_job(&profile, &excluded_job);

        assert!(clean_fit.score > excluded_fit.score);
        assert!(
            excluded_fit
                .reasons
                .iter()
                .any(|reason| reason.contains("Exclude term penalty applied"))
        );
    }

    #[test]
    fn explanations_include_positive_and_negative_reasons() {
        let service = SearchMatchingService::new();
        let profile = search_profile();
        let job = JobView {
            job: Job {
                seniority: Some("junior".to_string()),
                ..job_view(
                    "job-1",
                    "Backend Platform Engineer",
                    "Hybrid EU role with Rust and Postgres for a gambling platform",
                    Some("hybrid"),
                    "djinni",
                )
                .job
            },
            ..job_view(
                "job-1",
                "Backend Platform Engineer",
                "Hybrid EU role with Rust and Postgres for a gambling platform",
                Some("hybrid"),
                "djinni",
            )
        };

        let fit = service.score_job(&profile, &job);

        assert!(
            fit.reasons
                .iter()
                .any(|reason| reason.contains("Matched profile skills"))
        );
        assert!(
            fit.reasons
                .iter()
                .any(|reason| reason.contains("Exclude term penalty applied"))
        );
        assert!(
            fit.reasons
                .iter()
                .any(|reason| reason.contains("Work mode mismatch penalty applied"))
        );
        assert!(
            fit.reasons
                .iter()
                .any(|reason| reason.contains("Seniority mismatch penalty applied"))
        );
    }

    #[test]
    fn profile_aligned_jobs_rank_above_weakly_related_jobs() {
        let service = SearchMatchingService::new();
        let profile = search_profile();

        let exact_backend = job_view(
            "job-1",
            "Senior Backend Developer",
            "Remote EU role with Rust, Postgres, and backend platform work",
            Some("remote"),
            "djinni",
        );
        let partial_devops = job_view(
            "job-2",
            "Senior DevOps Engineer",
            "Remote EU platform role with AWS, docker, kubernetes, and backend API ownership",
            Some("remote"),
            "djinni",
        );
        let weak_match = job_view(
            "job-3",
            "Senior Product Manager",
            "Remote EU product strategy role with roadmap ownership",
            Some("remote"),
            "djinni",
        );

        let result = service.run(&profile, vec![weak_match, partial_devops, exact_backend]);

        assert_eq!(result.ranked_jobs[0].job.job.id, "job-1");
        assert_eq!(result.ranked_jobs[1].job.job.id, "job-2");
        assert_eq!(result.ranked_jobs[2].job.job.id, "job-3");
    }

    #[test]
    fn canonical_frontend_terms_survive_matching_and_explanations() {
        let service = SearchMatchingService::new();
        let profile = frontend_profile();
        let job = job_view(
            "job-frontend-1",
            "Senior Front-end React Developer",
            "Remote EU role shipping frontend design system work with React and TypeScript",
            Some("remote"),
            "djinni",
        );

        let fit = service.score_job(&profile, &job);

        assert!(fit.score >= 70);
        assert!(fit.matched_roles.contains(&RoleId::FrontendEngineer));
        assert!(fit.matched_skills.contains(&"react".to_string()));
        assert!(fit.matched_keywords.contains(&"frontend".to_string()));
        assert!(
            !fit.matched_keywords
                .iter()
                .any(|term| term == "front" || term == "end")
        );
        assert!(
            fit.reasons
                .iter()
                .any(|reason| reason.contains("Matched profile keywords: frontend"))
        );
    }

    #[test]
    fn react_native_matching_keeps_phrase_safe_internal_tokens_internal() {
        let service = SearchMatchingService::new();
        let profile = mobile_profile();
        let job = job_view(
            "job-mobile-1",
            "Senior React-Native Developer",
            "Remote EU role building React Native apps with TypeScript and distributed systems work",
            Some("remote"),
            "djinni",
        );

        let fit = service.score_job(&profile, &job);

        assert!(fit.score >= 70);
        assert!(fit.matched_roles.contains(&RoleId::MobileEngineer));
        assert!(fit.matched_skills.contains(&"react native".to_string()));
        assert!(!fit.matched_skills.iter().any(|term| term == "react_native"));
        assert!(
            fit.reasons
                .iter()
                .any(|reason| reason.contains("Matched profile skills: react native, typescript"))
        );
    }

    #[test]
    fn frontend_react_overlap_beats_generic_engineering_overlap() {
        let service = SearchMatchingService::new();
        let profile = frontend_profile();
        let strong_match = job_view(
            "job-frontend-strong",
            "Senior Front-end React Developer",
            "Remote EU role shipping frontend design system work with React and TypeScript",
            Some("remote"),
            "djinni",
        );
        let weak_match = job_view(
            "job-frontend-weak",
            "Senior UI Engineer",
            "Remote EU role improving shared product experiences and collaborating with design",
            Some("remote"),
            "djinni",
        );

        let strong_fit = service.score_job(&profile, &strong_match);
        let weak_fit = service.score_job(&profile, &weak_match);

        assert!(strong_fit.score > weak_fit.score);
        assert!(
            strong_fit
                .matched_keywords
                .contains(&"frontend".to_string())
        );
        assert!(strong_fit.matched_skills.contains(&"react".to_string()));
    }

    #[test]
    fn non_contiguous_frontend_search_phrase_matches_canonical_frontend_term() {
        let service = SearchMatchingService::new();
        let profile = SearchProfile {
            primary_role: RoleId::FrontendEngineer,
            primary_role_confidence: Some(96),
            target_roles: vec![RoleId::FrontendEngineer],
            role_candidates: vec![SearchRoleCandidate {
                role: RoleId::FrontendEngineer,
                confidence: 96,
            }],
            seniority: "senior".to_string(),
            target_regions: vec![TargetRegion::EuRemote],
            work_modes: vec![WorkMode::Remote],
            allowed_sources: vec![SourceId::Djinni],
            profile_skills: vec!["react".to_string()],
            profile_keywords: vec!["design system".to_string()],
            search_terms: vec!["frontend specialist".to_string()],
            exclude_terms: vec![],
        };
        let job = job_view(
            "job-frontend-search-term",
            "Senior Front-end React Developer",
            "Remote EU role shipping frontend design system work with React",
            Some("remote"),
            "djinni",
        );

        let fit = service.score_job(&profile, &job);

        assert!(fit.score >= 70);
        assert!(
            fit.reasons
                .iter()
                .any(|reason| reason.contains("Matched search terms: frontend"))
        );
    }

    #[test]
    fn backend_platform_overlap_prefers_engineering_stack_signals() {
        let service = SearchMatchingService::new();
        let profile = search_profile();
        let platform_job = job_view(
            "job-platform-1",
            "Senior Platform Engineer",
            "Remote EU platform role owning Rust APIs, Postgres, GraphQL, and distributed systems",
            Some("remote"),
            "djinni",
        );
        let generic_job = job_view(
            "job-generic-1",
            "Senior Software Engineer",
            "Remote EU role collaborating across product teams and improving internal tools",
            Some("remote"),
            "djinni",
        );

        let platform_fit = service.score_job(&profile, &platform_job);
        let generic_fit = service.score_job(&profile, &generic_job);

        assert!(platform_fit.score > generic_fit.score);
        assert!(platform_fit.matched_skills.contains(&"rust".to_string()));
        assert!(
            platform_fit
                .matched_skills
                .contains(&"postgres".to_string())
        );
        assert!(
            platform_fit
                .matched_keywords
                .contains(&"distributed systems".to_string())
        );
    }

    #[test]
    fn stale_job_scores_lower_than_fresh_identical_job() {
        let service = SearchMatchingService::new();
        let profile = search_profile();
        let fresh_job = job_view(
            "job-fresh",
            "Senior Backend Developer",
            "Remote EU role with Rust and Postgres",
            Some("remote"),
            "djinni",
        );
        // Job last seen 365 days ago — well past the 14-day freshness threshold.
        let stale_job = JobView {
            job: Job {
                last_seen_at: "2020-01-01T00:00:00Z".to_string(),
                ..fresh_job.job.clone()
            },
            ..fresh_job.clone()
        };

        let fresh_fit = service.score_job(&profile, &fresh_job);
        let stale_fit = service.score_job(&profile, &stale_job);

        assert!(
            fresh_fit.score > stale_fit.score,
            "fresh score {} should beat stale score {}",
            fresh_fit.score,
            stale_fit.score,
        );
        assert!(
            stale_fit
                .reasons
                .iter()
                .any(|r| r.contains("Freshness decay applied")),
            "stale job reasons should contain freshness decay explanation"
        );
    }

    #[test]
    fn missing_signals_stay_specific_and_drop_generic_noise() {
        let service = SearchMatchingService::new();
        let profile = frontend_profile();
        let weak_job = job_view(
            "job-frontend-gap",
            "Senior UI Engineer",
            "Remote EU role improving shared product experiences with design collaboration",
            Some("remote"),
            "djinni",
        );

        let fit = service.score_job(&profile, &weak_job);

        assert!(fit.missing_signals.contains(&"react".to_string()));
        assert!(fit.missing_signals.contains(&"typescript".to_string()));
        assert!(fit.missing_signals.contains(&"design system".to_string()));
        assert!(!fit.missing_signals.iter().any(|term| term == "developer"));
        assert!(!fit.missing_signals.iter().any(|term| term == "engineer"));
    }
}
