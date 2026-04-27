//! Deterministic profile/job fit scoring for single-entity flows.
//!
//! Prefer importing this logic through `crate::services::fit_scoring`.
//! It is intentionally narrower than `crate::services::search_ranking`:
//! fit scoring answers "how well does this one profile fit this one job?",
//! while search ranking ranks a list of jobs for a structured search profile.

pub mod runtime;

use crate::domain::candidate::profile::CandidateProfile;
use crate::domain::job::model::Job;
use crate::domain::profile::model::Profile;
use crate::domain::ranking::{FitScore, FitScoreComponents};
use crate::services::profile_analysis::rules::KNOWN_SKILLS;

// Static currency-to-USD conversion rates.
// These are rough mid-market rates for normalization only, not financial data.
const UAH_TO_USD: f64 = 1.0 / 41.0;
const EUR_TO_USD: f64 = 1.09;

#[derive(Clone, Default)]
pub struct RankingService;

impl RankingService {
    pub fn new() -> Self {
        Self
    }

    /// Compute a deterministic 0-100 fit score without any external API calls.
    ///
    /// `candidate` comes from analyzing the active resume via ProfileAnalysisService.
    /// `profile`   is the user's stored profile (optional — salary/workmode prefs).
    pub fn compute(
        &self,
        candidate: &CandidateProfile,
        job: &Job,
        profile: Option<&Profile>,
    ) -> FitScore {
        let job_description_lower = job.description_text.to_ascii_lowercase();

        // Extract skills mentioned in the job description using the same list used for CV parsing.
        let job_skills: Vec<String> = KNOWN_SKILLS
            .iter()
            .copied()
            .filter(|s| job_description_lower.contains(s))
            .map(|s| s.to_string())
            .collect();

        // Partition candidate skills into matched / missing against the job.
        let (matched_skills, missing_skills) = partition_skills(&candidate.skills, &job_skills);

        // --- Component 1: skill overlap (weight 40%) ---
        let skill_overlap = if candidate.skills.is_empty() {
            0.5
        } else {
            (matched_skills.len() as f32 / candidate.skills.len() as f32).min(1.0)
        };

        // --- Signal 2: seniority fit ---
        // Discrete score delta: +10 exact match, -3 for 1-level gap, -10 for ≥2-level gap.
        // 0 when job has no seniority or candidate seniority is unset.
        let seniority_fit_signal =
            compute_seniority_fit_signal(&candidate.seniority, job.seniority.as_deref());

        // --- Component 3: salary overlap (weight 15%) ---
        let (salary_min, salary_max, salary_currency) = profile
            .map(|p| (p.salary_min, p.salary_max, Some(p.salary_currency.as_str())))
            .unwrap_or((None, None, None));
        let salary_overlap = compute_salary_overlap(
            salary_min,
            salary_max,
            salary_currency,
            job.salary_min,
            job.salary_max,
            job.salary_currency.as_deref(),
        );

        // --- Signal 4: work mode preference ---
        // Applies a signed score delta based on the candidate's stated preference
        // vs the job's remote_type. 0 when no preference is set or pref is "any".
        let work_mode_signal = compute_work_mode_preference_signal(
            profile.and_then(|p| p.preferred_work_mode.as_deref()),
            job.remote_type.as_deref(),
        );

        // --- Signal 5: language preference ---
        // +5 match, -5 mismatch, 0 for bilingual job / no preference / unknown.
        let language_signal = compute_language_preference_signal(
            profile.and_then(|p| p.preferred_language.as_deref()),
            job.language.as_deref(),
        );

        // --- Component 6: recency bonus (weight 10%) ---
        // How fresh is the profile relative to when the job was posted?
        // A profile updated after (or close to) the posting date scores 1.0;
        // older profiles decay linearly to 0.0 at 180 days of lag.
        let recency_bonus = compute_recency_bonus(
            profile.and_then(|p| p.skills_updated_at.as_deref()),
            job.posted_at.as_deref(),
        );

        let base = (skill_overlap * 0.40
            + salary_overlap * 0.15
            + recency_bonus * 0.10)
            * 100.0;

        let total = (base
            + f32::from(work_mode_signal)
            + f32::from(seniority_fit_signal)
            + f32::from(language_signal))
        .clamp(0.0, 100.0)
        .round() as u8;

        FitScore {
            job_id: job.id.clone(),
            total,
            components: FitScoreComponents {
                skill_overlap,
                seniority_alignment: f32::from(seniority_fit_signal),
                salary_overlap,
                work_mode_match: f32::from(work_mode_signal),
                language_match: f32::from(language_signal),
                recency_bonus,
            },
            matched_skills,
            missing_skills,
        }
    }
}

/// Returns (matched, missing) partitions of `candidate_skills` against `job_skills`.
fn partition_skills(
    candidate_skills: &[String],
    job_skills: &[String],
) -> (Vec<String>, Vec<String>) {
    let job_lower: Vec<String> = job_skills.iter().map(|s| s.to_ascii_lowercase()).collect();

    let mut matched = Vec::new();
    let mut missing = Vec::new();

    for skill in candidate_skills {
        if job_lower.contains(&skill.to_ascii_lowercase()) {
            matched.push(skill.clone());
        } else {
            missing.push(skill.clone());
        }
    }

    (matched, missing)
}

fn seniority_ordinal(s: &str) -> Option<i32> {
    match s.to_lowercase().trim() {
        "intern" => Some(0),
        "junior" => Some(1),
        "middle" | "mid" | "mid-level" => Some(2),
        "senior" => Some(3),
        "lead" => Some(4),
        "staff" => Some(5),
        "principal" | "head" | "director" => Some(6),
        _ => None,
    }
}

/// Returns a discrete score delta for candidate-vs-job seniority fit.
///
/// | gap    | delta |
/// |--------|-------|
/// | 0      |  +10  |
/// | 1      |   -3  |
/// | ≥ 2    |  -10  |
/// | no job |    0  |
fn compute_seniority_fit_signal(candidate: &str, job: Option<&str>) -> i16 {
    let Some(job_str) = job else { return 0 };

    let (Some(c), Some(j)) = (seniority_ordinal(candidate), seniority_ordinal(job_str)) else {
        return 0;
    };

    match (c - j).unsigned_abs() {
        0 => 10,
        1 => -3,
        _ => -10,
    }
}

fn normalize_to_usd(amount: i32, currency: &str) -> f64 {
    let value = amount as f64;
    match currency.to_uppercase().trim() {
        "UAH" | "UAH/MONTH" => value * UAH_TO_USD,
        "EUR" => value * EUR_TO_USD,
        _ => value, // assume USD for unknown
    }
}

fn compute_salary_overlap(
    cand_min: Option<i32>,
    cand_max: Option<i32>,
    cand_currency: Option<&str>,
    job_min: Option<i32>,
    job_max: Option<i32>,
    job_currency: Option<&str>,
) -> f32 {
    let job_currency = job_currency.unwrap_or("USD");
    let candidate_currency = cand_currency.unwrap_or("USD");

    let (Some(j_min), Some(j_max)) = (job_min, job_max) else {
        return 0.5; // no job salary data → neutral
    };

    let (Some(c_min), Some(c_max)) = (cand_min, cand_max) else {
        return 0.5; // no candidate preference → neutral
    };

    let j_min_usd = normalize_to_usd(j_min, job_currency);
    let j_max_usd = normalize_to_usd(j_max, job_currency);
    let c_min_usd = normalize_to_usd(c_min, candidate_currency);
    let c_max_usd = normalize_to_usd(c_max, candidate_currency);

    let job_range = (j_max_usd - j_min_usd).max(1.0);
    let overlap = (c_max_usd.min(j_max_usd) - c_min_usd.max(j_min_usd)).max(0.0);

    (overlap / job_range).min(1.0) as f32
}

/// Returns a signed score delta based on the candidate's work mode preference
/// and the job's remote_type.
///
/// | preference  | job mode | delta |
/// |-------------|----------|-------|
/// | remote_only | remote   |  +8   |
/// | remote_only | onsite   | -10   |
/// | hybrid      | remote   |  +3   |
/// | hybrid      | onsite   |  -3   |
/// | any         | *        |   0   |
/// | None        | *        |   0   |
fn compute_work_mode_preference_signal(pref: Option<&str>, job_mode: Option<&str>) -> i16 {
    let Some(pref_str) = pref else {
        return 0;
    };

    let pref_norm = normalize_candidate_work_mode(pref_str);

    if matches!(pref_norm.as_deref(), Some("any") | None) {
        return 0;
    }

    let Some(job_norm) = job_mode.and_then(normalize_job_work_mode_for_fit) else {
        return 0;
    };

    match (pref_norm.as_deref(), job_norm.as_str()) {
        (Some("remote"), "remote") => 8,
        (Some("remote"), "onsite") => -10,
        (Some("hybrid"), "remote") => 3,
        (Some("hybrid"), "onsite") => -3,
        _ => 0,
    }
}

fn normalize_candidate_work_mode(s: &str) -> Option<String> {
    match s.to_lowercase().trim() {
        "remote" | "remote_only" | "full_remote" | "fully_remote" | "fully remote"
        | "full remote" => Some("remote".to_string()),
        "hybrid" => Some("hybrid".to_string()),
        "onsite" | "on-site" | "office" | "in-office" => Some("onsite".to_string()),
        "any" => Some("any".to_string()),
        _ => None,
    }
}

fn normalize_job_work_mode_for_fit(s: &str) -> Option<String> {
    match s.to_lowercase().trim() {
        "remote" | "full_remote" | "fully_remote" | "fully remote" | "full remote" => {
            Some("remote".to_string())
        }
        "hybrid" => Some("hybrid".to_string()),
        "onsite" | "on-site" | "office" | "in-office" => Some("onsite".to_string()),
        _ => None,
    }
}

/// Returns a signed score delta based on the candidate's preferred job language
/// and the language field on the job posting.
///
/// | profile pref  | job language | delta |
/// |---------------|--------------|-------|
/// | Ukrainian     | Ukrainian    |  +5   |
/// | English       | English      |  +5   |
/// | Ukrainian     | English      |  -5   |
/// | English       | Ukrainian    |  -5   |
/// | bilingual     | *            |   0   |
/// | any           | *            |   0   |
/// | None          | *            |   0   |
/// | *             | bilingual    |   0   |
/// | *             | None/unknown |   0   |
fn compute_language_preference_signal(pref: Option<&str>, job_language: Option<&str>) -> i16 {
    let Some(pref_str) = pref else { return 0 };

    let pref_norm = normalize_language_pref(pref_str);
    if matches!(pref_norm.as_deref(), Some("bilingual") | Some("any") | None) {
        return 0;
    }

    let Some(job_lang) = job_language.and_then(normalize_job_language) else {
        return 0;
    };
    if job_lang == "bilingual" {
        return 0;
    }

    if pref_norm.as_deref() == Some(job_lang.as_str()) {
        5
    } else {
        -5
    }
}

fn normalize_language_pref(s: &str) -> Option<String> {
    match s.to_lowercase().trim() {
        "ukrainian" | "uk" => Some("ukrainian".to_string()),
        "english" | "en" => Some("english".to_string()),
        "bilingual" => Some("bilingual".to_string()),
        "any" => Some("any".to_string()),
        _ => None,
    }
}

fn normalize_job_language(s: &str) -> Option<String> {
    match s.to_lowercase().trim() {
        "ukrainian" | "uk" => Some("ukrainian".to_string()),
        "english" | "en" => Some("english".to_string()),
        "bilingual" => Some("bilingual".to_string()),
        _ => None,
    }
}

/// Convert an ISO date string ("YYYY-MM-DD" or "YYYY-MM-DDTHH:…") to a Julian
/// Day Number. Uses the proleptic Gregorian formula; accurate to ±1 day for
/// years 2000-2100 which is all we need for recency calculations.
fn parse_date_to_jdn(date_str: &str) -> Option<i64> {
    let date = date_str.get(..10)?; // take "YYYY-MM-DD"
    let y: i64 = date.get(..4)?.parse().ok()?;
    let m: i64 = date.get(5..7)?.parse().ok()?;
    let d: i64 = date.get(8..10)?.parse().ok()?;

    // Standard Julian Day Number formula (integer arithmetic only).
    let a = (14 - m) / 12;
    let yy = y + 4800 - a;
    let mm = m + 12 * a - 3;
    Some(d + (153 * mm + 2) / 5 + 365 * yy + yy / 4 - yy / 100 + yy / 400 - 32045)
}

/// Score how fresh the candidate's profile is relative to when the job was posted.
///
/// - Profile updated on or after the job posting  → 1.0 (fully current)
/// - 180+ days before the job was posted          → 0.0 (stale)
/// - Linear interpolation in between
/// - Either date absent or unparseable             → 0.5 (neutral)
fn compute_recency_bonus(profile_updated_at: Option<&str>, job_posted_at: Option<&str>) -> f32 {
    let (Some(profile_date), Some(job_date)) = (profile_updated_at, job_posted_at) else {
        return 0.5;
    };

    let (Some(profile_jdn), Some(job_jdn)) =
        (parse_date_to_jdn(profile_date), parse_date_to_jdn(job_date))
    else {
        return 0.5;
    };

    // Positive lag means profile is older than the job posting.
    let lag_days = (job_jdn - profile_jdn).max(0) as f32;
    (1.0 - lag_days / 180.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use crate::domain::candidate::profile::{CandidateProfile, RoleScore};
    use crate::domain::job::model::Job;
    use crate::domain::profile::model::Profile;
    use crate::domain::role::RoleId;

    use super::RankingService;

    fn make_job(
        skills_in_desc: &str,
        seniority: Option<&str>,
        salary_min: Option<i32>,
        salary_max: Option<i32>,
        currency: Option<&str>,
        remote_type: Option<&str>,
    ) -> Job {
        Job {
            id: "job-1".to_string(),
            title: "Engineer".to_string(),
            company_name: "Acme".to_string(),
            location: None,
            remote_type: remote_type.map(str::to_string),
            seniority: seniority.map(str::to_string),
            description_text: skills_in_desc.to_string(),
            salary_min,
            salary_max,
            salary_currency: currency.map(str::to_string),
            language: None,
            posted_at: None,
            last_seen_at: "2026-04-12".to_string(),
            is_active: true,
        }
    }

    fn make_candidate(skills: &[&str], seniority: &str) -> CandidateProfile {
        CandidateProfile {
            summary: String::new(),
            primary_role: RoleId::FrontendEngineer,
            seniority: seniority.to_string(),
            skills: skills.iter().map(|s| s.to_string()).collect(),
            keywords: vec![],
            role_candidates: vec![RoleScore {
                role: RoleId::FrontendEngineer,
                score: 10,
                confidence: 80,
                matched_signals: vec![],
            }],
            suggested_search_terms: vec![],
        }
    }

    #[test]
    fn perfect_skill_match_scores_high() {
        let service = RankingService::new();
        let candidate = make_candidate(&["react", "typescript"], "senior");
        let job = make_job(
            "We need react and typescript developers",
            Some("senior"),
            None,
            None,
            None,
            None,
        );

        let score = service.compute(&candidate, &job, None);

        assert!(
            score.total >= 60,
            "perfect skill + seniority match should score >= 60, got {}",
            score.total
        );
        assert_eq!(score.matched_skills, vec!["react", "typescript"]);
        assert!(score.missing_skills.is_empty());
    }

    #[test]
    fn no_skill_overlap_scores_lower() {
        let service = RankingService::new();
        let candidate = make_candidate(&["react", "typescript"], "senior");
        let job = make_job(
            "We need python and java developers",
            Some("senior"),
            None,
            None,
            None,
            None,
        );

        let score = service.compute(&candidate, &job, None);

        // Skill overlap = 0, but seniority match + recency neutral push it above 0.
        assert!(
            score.total < 50,
            "no skill match should score < 50, got {}",
            score.total
        );
        assert!(score.matched_skills.is_empty());
    }

    #[test]
    fn seniority_mismatch_penalizes_score() {
        let service = RankingService::new();
        let candidate = make_candidate(&["react"], "junior");
        let job_senior = make_job(
            "react developer needed",
            Some("senior"),
            None,
            None,
            None,
            None,
        );
        let job_junior = make_job(
            "react developer needed",
            Some("junior"),
            None,
            None,
            None,
            None,
        );

        let score_senior = service.compute(&candidate, &job_senior, None);
        let score_junior = service.compute(&candidate, &job_junior, None);

        assert!(
            score_junior.total > score_senior.total,
            "junior→junior should score higher than junior→senior"
        );
    }

    #[test]
    fn salary_overlap_normalizes_candidate_currency() {
        let overlap = super::compute_salary_overlap(
            Some(120_000),
            Some(160_000),
            Some("UAH"),
            Some(3_000),
            Some(3_800),
            Some("USD"),
        );

        assert!(
            overlap > 0.5,
            "candidate salary should be normalized from UAH before overlap, got {overlap}"
        );
    }

    #[test]
    fn unknown_seniority_gives_zero_signal() {
        let service = RankingService::new();
        let candidate = make_candidate(&[], "unknown");
        let job = make_job("some job description", None, None, None, None, None);

        let score = service.compute(&candidate, &job, None);
        assert_eq!(score.components.seniority_alignment, 0.0);
    }

    #[test]
    fn recency_bonus_is_max_when_profile_is_current() {
        // Profile updated same day as the job posting → 1.0.
        let bonus = super::compute_recency_bonus(Some("2026-04-01"), Some("2026-04-01"));
        assert!(
            (bonus - 1.0).abs() < 0.01,
            "same-day profile should give recency 1.0, got {bonus}"
        );
    }

    #[test]
    fn recency_bonus_decays_linearly() {
        // 90-day lag → 0.5
        let bonus = super::compute_recency_bonus(Some("2026-01-01"), Some("2026-04-01"));
        assert!(
            (bonus - 0.5).abs() < 0.02,
            "90-day lag should give ~0.5, got {bonus}"
        );
    }

    #[test]
    fn recency_bonus_is_zero_when_profile_is_stale() {
        // 180+ day lag → 0.0
        let bonus = super::compute_recency_bonus(Some("2025-10-01"), Some("2026-04-01"));
        assert!(
            bonus <= 0.01,
            "180+ day lag should give recency ~0.0, got {bonus}"
        );
    }

    #[test]
    fn recency_bonus_is_neutral_when_dates_are_absent() {
        let bonus = super::compute_recency_bonus(None, None);
        assert!(
            (bonus - 0.5).abs() < 0.01,
            "absent dates should give recency 0.5, got {bonus}"
        );
    }

    #[test]
    fn compute_uses_skills_timestamp_instead_of_generic_profile_update_time() {
        let service = RankingService::new();
        let candidate = make_candidate(&["rust"], "senior");
        let mut job = make_job(
            "rust engineer needed",
            Some("senior"),
            None,
            None,
            None,
            Some("remote"),
        );
        job.posted_at = Some("2026-04-01".to_string());

        let profile = Profile {
            id: "profile-1".to_string(),
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            location: None,
            raw_text: "Senior Rust engineer".to_string(),
            analysis: None,
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_work_mode: Some("remote".to_string()),
            preferred_language: None,
            search_preferences: None,
            created_at: "2026-01-01".to_string(),
            updated_at: "2026-04-01".to_string(),
            skills_updated_at: Some("2026-01-01".to_string()),
        };

        let score = service.compute(&candidate, &job, Some(&profile));

        assert!(
            score.components.recency_bonus < 0.6,
            "recency should follow stale skills timestamp, got {}",
            score.components.recency_bonus
        );
    }

    // --- work mode preference signal tests ---

    #[test]
    fn work_mode_remote_only_plus_remote_job_gives_plus_8() {
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("remote_only"), Some("remote")),
            8
        );
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("remote"), Some("remote")),
            8
        );
    }

    #[test]
    fn work_mode_remote_only_plus_onsite_job_gives_minus_10() {
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("remote_only"), Some("onsite")),
            -10
        );
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("remote"), Some("onsite")),
            -10
        );
    }

    #[test]
    fn work_mode_hybrid_plus_remote_job_gives_plus_3() {
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("hybrid"), Some("remote")),
            3
        );
    }

    #[test]
    fn work_mode_hybrid_plus_onsite_job_gives_minus_3() {
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("hybrid"), Some("onsite")),
            -3
        );
    }

    #[test]
    fn work_mode_any_preference_gives_zero() {
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("any"), Some("remote")),
            0
        );
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("any"), Some("onsite")),
            0
        );
    }

    #[test]
    fn work_mode_no_preference_gives_zero() {
        assert_eq!(
            super::compute_work_mode_preference_signal(None, Some("remote")),
            0
        );
        assert_eq!(
            super::compute_work_mode_preference_signal(None, Some("onsite")),
            0
        );
    }

    #[test]
    fn work_mode_unknown_job_mode_gives_zero() {
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("remote"), Some("unknown_mode")),
            0
        );
        assert_eq!(
            super::compute_work_mode_preference_signal(Some("hybrid"), None),
            0
        );
    }

    // --- seniority fit signal tests ---

    #[test]
    fn seniority_exact_match_gives_plus_10() {
        assert_eq!(
            super::compute_seniority_fit_signal("senior", Some("senior")),
            10
        );
        assert_eq!(
            super::compute_seniority_fit_signal("junior", Some("junior")),
            10
        );
        assert_eq!(
            super::compute_seniority_fit_signal("lead", Some("lead")),
            10
        );
    }

    #[test]
    fn seniority_one_level_gap_gives_minus_3() {
        assert_eq!(
            super::compute_seniority_fit_signal("senior", Some("lead")),
            -3
        );
        assert_eq!(
            super::compute_seniority_fit_signal("senior", Some("middle")),
            -3
        );
        assert_eq!(
            super::compute_seniority_fit_signal("junior", Some("middle")),
            -3
        );
    }

    #[test]
    fn seniority_two_level_gap_gives_minus_10() {
        assert_eq!(
            super::compute_seniority_fit_signal("junior", Some("senior")),
            -10
        );
        assert_eq!(
            super::compute_seniority_fit_signal("senior", Some("junior")),
            -10
        );
        assert_eq!(
            super::compute_seniority_fit_signal("junior", Some("lead")),
            -10
        );
    }

    #[test]
    fn seniority_no_job_seniority_gives_zero() {
        assert_eq!(
            super::compute_seniority_fit_signal("senior", None),
            0
        );
        assert_eq!(
            super::compute_seniority_fit_signal("junior", None),
            0
        );
    }

    #[test]
    fn seniority_unknown_profile_seniority_gives_zero() {
        assert_eq!(
            super::compute_seniority_fit_signal("unknown", Some("senior")),
            0
        );
        assert_eq!(
            super::compute_seniority_fit_signal("", Some("junior")),
            0
        );
    }

    #[test]
    fn work_mode_signal_affects_total_score() {
        let service = RankingService::new();
        let candidate = make_candidate(&["rust"], "senior");
        let remote_job = make_job("rust engineer", Some("senior"), None, None, None, Some("remote"));
        let onsite_job = make_job("rust engineer", Some("senior"), None, None, None, Some("onsite"));

        let profile_remote = Profile {
            id: "p1".to_string(),
            name: "Dev".to_string(),
            email: "dev@example.com".to_string(),
            location: None,
            raw_text: String::new(),
            analysis: None,
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_work_mode: Some("remote_only".to_string()),
            preferred_language: None,
            search_preferences: None,
            created_at: "2026-01-01".to_string(),
            updated_at: "2026-01-01".to_string(),
            skills_updated_at: None,
        };

        let score_remote = service.compute(&candidate, &remote_job, Some(&profile_remote));
        let score_onsite = service.compute(&candidate, &onsite_job, Some(&profile_remote));

        assert!(
            score_remote.total > score_onsite.total,
            "remote_only pref + remote job ({}) should score higher than remote_only pref + onsite job ({})",
            score_remote.total,
            score_onsite.total,
        );
        assert_eq!(score_remote.components.work_mode_match, 8.0);
        assert_eq!(score_onsite.components.work_mode_match, -10.0);
    }

    // --- language preference signal tests ---

    #[test]
    fn language_match_gives_plus_5() {
        assert_eq!(
            super::compute_language_preference_signal(Some("Ukrainian"), Some("Ukrainian")),
            5
        );
        assert_eq!(
            super::compute_language_preference_signal(Some("English"), Some("English")),
            5
        );
    }

    #[test]
    fn language_mismatch_gives_minus_5() {
        assert_eq!(
            super::compute_language_preference_signal(Some("Ukrainian"), Some("English")),
            -5
        );
        assert_eq!(
            super::compute_language_preference_signal(Some("English"), Some("Ukrainian")),
            -5
        );
    }

    #[test]
    fn bilingual_job_gives_zero_signal() {
        assert_eq!(
            super::compute_language_preference_signal(Some("Ukrainian"), Some("bilingual")),
            0
        );
        assert_eq!(
            super::compute_language_preference_signal(Some("English"), Some("bilingual")),
            0
        );
    }

    #[test]
    fn bilingual_preference_gives_zero_signal() {
        assert_eq!(
            super::compute_language_preference_signal(Some("bilingual"), Some("Ukrainian")),
            0
        );
        assert_eq!(
            super::compute_language_preference_signal(Some("bilingual"), Some("English")),
            0
        );
    }

    #[test]
    fn any_preference_gives_zero_signal() {
        assert_eq!(
            super::compute_language_preference_signal(Some("any"), Some("Ukrainian")),
            0
        );
        assert_eq!(
            super::compute_language_preference_signal(Some("any"), Some("English")),
            0
        );
    }

    #[test]
    fn no_preference_gives_zero_signal() {
        assert_eq!(
            super::compute_language_preference_signal(None, Some("Ukrainian")),
            0
        );
        assert_eq!(
            super::compute_language_preference_signal(None, Some("English")),
            0
        );
    }

    #[test]
    fn no_job_language_gives_zero_signal() {
        assert_eq!(
            super::compute_language_preference_signal(Some("Ukrainian"), None),
            0
        );
        assert_eq!(
            super::compute_language_preference_signal(Some("English"), None),
            0
        );
    }

    #[test]
    fn language_signal_affects_total_score() {
        let service = RankingService::new();
        let candidate = make_candidate(&["rust"], "senior");

        let mut ukrainian_job = make_job("rust engineer", Some("senior"), None, None, None, None);
        ukrainian_job.language = Some("Ukrainian".to_string());
        let mut english_job = make_job("rust engineer", Some("senior"), None, None, None, None);
        english_job.language = Some("English".to_string());

        let profile_ua = Profile {
            id: "p-ua".to_string(),
            name: "Dev".to_string(),
            email: "dev@example.com".to_string(),
            location: None,
            raw_text: String::new(),
            analysis: None,
            years_of_experience: None,
            salary_min: None,
            salary_max: None,
            salary_currency: "USD".to_string(),
            languages: vec![],
            preferred_work_mode: None,
            preferred_language: Some("Ukrainian".to_string()),
            search_preferences: None,
            created_at: "2026-01-01".to_string(),
            updated_at: "2026-01-01".to_string(),
            skills_updated_at: None,
        };

        let score_ua_match = service.compute(&candidate, &ukrainian_job, Some(&profile_ua));
        let score_ua_mismatch = service.compute(&candidate, &english_job, Some(&profile_ua));

        assert_eq!(score_ua_match.components.language_match, 5.0);
        assert_eq!(score_ua_mismatch.components.language_match, -5.0);
        assert!(
            score_ua_match.total > score_ua_mismatch.total,
            "Ukrainian pref + Ukrainian job ({}) should score higher than Ukrainian pref + English job ({})",
            score_ua_match.total,
            score_ua_mismatch.total,
        );
    }
}
