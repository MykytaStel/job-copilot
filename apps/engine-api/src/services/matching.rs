use crate::domain::job::model::JobView;
use crate::domain::matching::JobFit;
use crate::domain::role::RoleId;
use crate::domain::search::profile::{SearchProfile, TargetRegion, WorkMode};
use crate::domain::source::SourceId;
use crate::services::profile::matching::{PreparedText, normalize_text};
use crate::services::profile::rules::KNOWN_SKILLS;

const ROLE_WEIGHT: f32 = 45.0;
const SEARCH_TERM_WEIGHT: f32 = 30.0;
const SOURCE_WEIGHT: f32 = 10.0;
const WORK_MODE_WEIGHT: f32 = 10.0;
const REGION_WEIGHT: f32 = 5.0;
const EXCLUDE_PENALTY_WEIGHT: f32 = 20.0;

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
}

#[derive(Clone, Default)]
pub struct SearchMatchingService;

impl SearchMatchingService {
    pub fn new() -> Self {
        Self
    }

    pub fn run(
        &self,
        search_profile: &SearchProfile,
        jobs: Vec<JobView>,
        limit: usize,
    ) -> SearchRunResult {
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
        ranked_jobs.truncate(limit);

        SearchRunResult {
            ranked_jobs,
            total_candidates,
            filtered_out_by_source,
        }
    }

    pub fn score_job(&self, search_profile: &SearchProfile, job: &JobView) -> JobFit {
        let prepared_text = PreparedText::new(&build_searchable_text(job));
        let target_roles = collect_target_roles(search_profile);
        let matched_roles = collect_matched_roles(&prepared_text, &target_roles);
        let role_terms = collect_role_terms(&target_roles);
        let matched_search_terms =
            collect_matched_terms(&prepared_text, &search_profile.search_terms, &role_terms);
        let searchable_terms = normalized_terms(&search_profile.search_terms)
            .into_iter()
            .filter(|term| !role_terms.contains(term))
            .collect::<Vec<_>>();
        let matched_skills = matched_search_terms
            .iter()
            .filter(|term| is_known_skill(term))
            .cloned()
            .collect::<Vec<_>>();
        let matched_keywords = matched_search_terms
            .iter()
            .filter(|term| !is_known_skill(term))
            .cloned()
            .collect::<Vec<_>>();
        let source_match = compute_source_match(search_profile, job);
        let work_mode_match = compute_work_mode_match(search_profile, job);
        let region_match = compute_region_match(search_profile, &prepared_text, job);
        let matched_exclude_terms =
            collect_matched_terms(&prepared_text, &search_profile.exclude_terms, &Vec::new());

        let role_score = overlap_ratio(matched_roles.len(), target_roles.len()) * ROLE_WEIGHT;
        let search_term_score =
            overlap_ratio(matched_search_terms.len(), searchable_terms.len()) * SEARCH_TERM_WEIGHT;
        let source_score = if source_match { SOURCE_WEIGHT } else { 0.0 };
        let work_mode_score = match work_mode_match {
            Some(true) => WORK_MODE_WEIGHT,
            Some(false) => 0.0,
            None if search_profile.work_modes.is_empty() => WORK_MODE_WEIGHT / 2.0,
            None => WORK_MODE_WEIGHT / 4.0,
        };
        let region_score = match region_match {
            Some(true) => REGION_WEIGHT,
            Some(false) => 0.0,
            None if search_profile.target_regions.is_empty() => REGION_WEIGHT / 2.0,
            None => REGION_WEIGHT / 4.0,
        };
        let exclude_penalty = overlap_ratio(
            matched_exclude_terms.len(),
            normalized_terms(&search_profile.exclude_terms).len(),
        ) * EXCLUDE_PENALTY_WEIGHT;

        let score = (role_score + search_term_score + source_score + work_mode_score + region_score
            - exclude_penalty)
            .clamp(0.0, 100.0)
            .round() as u8;

        JobFit {
            job_id: job.job.id.clone(),
            score,
            matched_roles,
            matched_skills,
            matched_keywords,
            source_match,
            work_mode_match,
            region_match,
            reasons: build_reasons(
                &target_roles,
                &search_profile.search_terms,
                &search_profile.allowed_sources,
                &search_profile.work_modes,
                &search_profile.target_regions,
                job,
                &matched_search_terms,
                &matched_exclude_terms,
                work_mode_match,
                region_match,
            ),
        }
    }
}

fn build_searchable_text(job: &JobView) -> String {
    let mut parts = vec![
        job.job.title.as_str(),
        job.job.company_name.as_str(),
        job.job.description_text.as_str(),
    ];

    if let Some(remote_type) = job.job.remote_type.as_deref() {
        parts.push(remote_type);
    }

    if let Some(source) = job
        .primary_variant
        .as_ref()
        .map(|variant| variant.source.as_str())
    {
        parts.push(source);
    }

    parts.join(" ")
}

fn collect_target_roles(search_profile: &SearchProfile) -> Vec<RoleId> {
    let mut roles = Vec::new();
    push_unique_role(&mut roles, search_profile.primary_role);

    for role in &search_profile.target_roles {
        push_unique_role(&mut roles, *role);
    }

    roles
}

fn collect_matched_roles(prepared_text: &PreparedText, target_roles: &[RoleId]) -> Vec<RoleId> {
    let mut matched_roles = Vec::new();

    for role in target_roles {
        if role_matches(prepared_text, *role) {
            push_unique_role(&mut matched_roles, *role);
        }
    }

    matched_roles
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

fn collect_matched_terms(
    prepared_text: &PreparedText,
    terms: &[String],
    ignored_terms: &[String],
) -> Vec<String> {
    let mut matched_terms = Vec::new();

    for term in terms {
        let normalized = normalize_text(term);

        if normalized.is_empty() || ignored_terms.contains(&normalized) {
            continue;
        }

        if prepared_text.matches_signal(&normalized) {
            push_unique_string(&mut matched_terms, normalized);
        }
    }

    matched_terms
}

fn normalized_terms(terms: &[String]) -> Vec<String> {
    let mut normalized_terms = Vec::new();

    for term in terms {
        let normalized = normalize_text(term);

        if normalized.is_empty() {
            continue;
        }

        push_unique_string(&mut normalized_terms, normalized);
    }

    normalized_terms
}

fn is_known_skill(term: &str) -> bool {
    KNOWN_SKILLS
        .iter()
        .any(|skill| normalize_text(skill) == normalize_text(term))
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

fn build_reasons(
    target_roles: &[RoleId],
    search_terms: &[String],
    allowed_sources: &[SourceId],
    work_modes: &[WorkMode],
    target_regions: &[TargetRegion],
    job: &JobView,
    matched_search_terms: &[String],
    matched_exclude_terms: &[String],
    work_mode_match: Option<bool>,
    region_match: Option<bool>,
) -> Vec<String> {
    let mut reasons = Vec::new();
    let matched_roles = collect_matched_roles(
        &PreparedText::new(&build_searchable_text(job)),
        target_roles,
    );

    if matched_roles.is_empty() {
        reasons.push("No target role signals matched the job title or description".to_string());
    } else {
        reasons.push(format!(
            "Matched target roles: {}",
            matched_roles
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if search_terms.is_empty() {
        reasons.push(
            "Search profile has no search terms; score leans on roles and filters".to_string(),
        );
    } else if matched_search_terms.is_empty() {
        reasons.push("No non-role search terms matched the job text".to_string());
    } else {
        reasons.push(format!(
            "Matched search terms: {}",
            matched_search_terms.join(", ")
        ));
    }

    if allowed_sources.is_empty() {
        reasons.push("All sources are allowed".to_string());
    } else if let Some(source) = job_source(job) {
        reasons.push(format!(
            "Source matched allowed sources: {}",
            source.canonical_key()
        ));
    } else {
        reasons.push("Job source was unavailable".to_string());
    }

    if !work_modes.is_empty() {
        match work_mode_match {
            Some(true) => reasons.push("Work mode matched the search profile".to_string()),
            Some(false) => reasons.push("Work mode did not match the search profile".to_string()),
            None => reasons.push("Work mode could not be inferred from the job".to_string()),
        }
    }

    if !target_regions.is_empty() {
        match region_match {
            Some(true) => reasons.push("Target region matched the job text".to_string()),
            Some(false) => reasons.push("Target region did not match the job text".to_string()),
            None => reasons.push("Region could not be inferred from the job".to_string()),
        }
    }

    if !matched_exclude_terms.is_empty() {
        reasons.push(format!(
            "Matched exclude terms: {}",
            matched_exclude_terms.join(", ")
        ));
    }

    reasons
}

fn overlap_ratio(matched: usize, total: usize) -> f32 {
    if total == 0 {
        0.0
    } else {
        (matched as f32 / total as f32).min(1.0)
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

#[cfg(test)]
mod tests {
    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
    use crate::domain::role::RoleId;
    use crate::domain::search::profile::{SearchProfile, TargetRegion, WorkMode};
    use crate::domain::source::SourceId;

    use super::SearchMatchingService;

    fn search_profile() -> SearchProfile {
        SearchProfile {
            primary_role: RoleId::BackendDeveloper,
            target_roles: vec![RoleId::BackendDeveloper, RoleId::DevopsEngineer],
            seniority: "senior".to_string(),
            target_regions: vec![TargetRegion::EuRemote],
            work_modes: vec![WorkMode::Remote],
            allowed_sources: vec![SourceId::Djinni],
            search_terms: vec![
                "rust".to_string(),
                "postgres".to_string(),
                "distributed systems".to_string(),
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
                .contains(&RoleId::BackendDeveloper)
        );
        assert!(matching_fit.matched_keywords.contains(&"rust".to_string()));
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
            10,
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
            10,
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
    fn explanations_are_included_in_fit_result() {
        let service = SearchMatchingService::new();
        let profile = search_profile();
        let job = job_view(
            "job-1",
            "Senior Backend Developer",
            "Remote EU role with Rust and Postgres for a gambling platform",
            Some("remote"),
            "djinni",
        );

        let fit = service.score_job(&profile, &job);

        assert!(
            fit.reasons
                .iter()
                .any(|reason| reason.contains("Matched target roles"))
        );
        assert!(
            fit.reasons
                .iter()
                .any(|reason| reason.contains("Matched exclude terms"))
        );
    }
}
