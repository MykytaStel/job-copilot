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

        // --- Component 2: seniority alignment (weight 25%) ---
        let seniority_alignment =
            compute_seniority_alignment(&candidate.seniority, job.seniority.as_deref());

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

        // --- Component 4: work mode match (weight 10%) ---
        let preferred_work_mode = profile.and_then(|p| p.preferred_work_mode.as_deref());
        let work_mode_match =
            compute_work_mode_match(preferred_work_mode, job.remote_type.as_deref());

        // --- Component 5: recency bonus (weight 10%) ---
        // How fresh is the profile relative to when the job was posted?
        // A profile updated after (or close to) the posting date scores 1.0;
        // older profiles decay linearly to 0.0 at 180 days of lag.
        let recency_bonus = compute_recency_bonus(
            profile.and_then(|p| p.skills_updated_at.as_deref()),
            job.posted_at.as_deref(),
        );

        let total = (skill_overlap * 0.40
            + seniority_alignment * 0.25
            + salary_overlap * 0.15
            + work_mode_match * 0.10
            + recency_bonus * 0.10)
            * 100.0;

        FitScore {
            job_id: job.id.clone(),
            total: total.round().clamp(0.0, 100.0) as u8,
            components: FitScoreComponents {
                skill_overlap,
                seniority_alignment,
                salary_overlap,
                work_mode_match,
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

/// Maps seniority strings to an ordinal level for gap calculation.
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

fn compute_seniority_alignment(candidate: &str, job: Option<&str>) -> f32 {
    let Some(job_level) = job else { return 0.5 };

    match (seniority_ordinal(candidate), seniority_ordinal(job_level)) {
        (Some(c), Some(j)) => {
            let gap = (c - j).unsigned_abs() as f32;
            // Perfect match = 1.0; each level of gap costs 0.33; gap ≥ 3 = 0.0.
            (1.0 - gap / 3.0).max(0.0)
        }
        _ => 0.5, // unknown seniority on either side → neutral
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

fn compute_work_mode_match(candidate_pref: Option<&str>, job_mode: Option<&str>) -> f32 {
    let (Some(pref), Some(mode)) = (candidate_pref, job_mode) else {
        return 0.5; // either side unspecified → neutral
    };

    let pref_norm = normalize_work_mode(pref);
    let mode_norm = normalize_work_mode(mode);

    match (pref_norm.as_deref(), mode_norm.as_deref()) {
        (Some(p), Some(m)) if p == m => 1.0,
        (Some("remote"), Some("hybrid")) | (Some("hybrid"), Some("remote")) => 0.5,
        (Some("hybrid"), Some("onsite")) | (Some("onsite"), Some("hybrid")) => 0.5,
        (Some("remote"), Some("onsite")) | (Some("onsite"), Some("remote")) => 0.0,
        _ => 0.5,
    }
}

fn normalize_work_mode(s: &str) -> Option<String> {
    match s.to_lowercase().trim() {
        "remote" | "full_remote" | "fully_remote" | "fully remote" | "full remote" => {
            Some("remote".to_string())
        }
        "hybrid" => Some("hybrid".to_string()),
        "onsite" | "on-site" | "office" | "in-office" => Some("onsite".to_string()),
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
            score.total >= 70,
            "perfect skill + seniority match should score >= 70, got {}",
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
    fn unknown_seniority_gives_neutral_component() {
        let service = RankingService::new();
        let candidate = make_candidate(&[], "unknown");
        let job = make_job("some job description", None, None, None, None, None);

        let score = service.compute(&candidate, &job, None);
        let expected_seniority = 0.5_f32;
        assert!((score.components.seniority_alignment - expected_seniority).abs() < 0.01);
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
}
