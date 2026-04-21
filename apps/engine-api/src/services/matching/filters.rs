use super::{
    PreparedText, SENIORITY_MISMATCH_PENALTY_WEIGHT, SENIORITY_WEIGHT, SearchProfile,
    SeniorityAlignment, SourceId, TargetRegion, WorkMode, normalize_text, push_unique_region,
};

pub(super) fn compute_source_match(search_profile: &SearchProfile, job: &super::JobView) -> bool {
    if search_profile.allowed_sources.is_empty() {
        return true;
    }

    let Some(job_source) = job_source(job) else {
        return false;
    };

    search_profile.allowed_sources.contains(&job_source)
}

pub(super) fn is_job_allowed_by_source(
    search_profile: &SearchProfile,
    job: &super::JobView,
) -> bool {
    if search_profile.allowed_sources.is_empty() {
        return true;
    }

    compute_source_match(search_profile, job)
}

pub(super) fn job_source(job: &super::JobView) -> Option<SourceId> {
    job.primary_variant
        .as_ref()
        .and_then(|variant| SourceId::parse_canonical_key(&variant.source))
}

pub(super) fn compute_work_mode_match(
    search_profile: &SearchProfile,
    job: &super::JobView,
) -> Option<bool> {
    if search_profile.work_modes.is_empty() {
        return None;
    }

    let job_mode = normalize_job_work_mode(job.job.remote_type.as_deref())?;
    Some(search_profile.work_modes.contains(&job_mode))
}

pub(super) fn normalize_job_work_mode(remote_type: Option<&str>) -> Option<WorkMode> {
    let value = normalize_text(remote_type?);

    match value.as_str() {
        "remote" | "full remote" | "fully remote" => Some(WorkMode::Remote),
        "hybrid" => Some(WorkMode::Hybrid),
        "onsite" | "on site" | "office" | "in office" => Some(WorkMode::Onsite),
        _ => None,
    }
}

pub(super) fn compute_region_match(
    search_profile: &SearchProfile,
    prepared_text: &PreparedText,
    job: &super::JobView,
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

fn detect_job_regions(prepared_text: &PreparedText, job: &super::JobView) -> Vec<TargetRegion> {
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

pub(super) fn matches_any(prepared_text: &PreparedText, signals: &[&str]) -> bool {
    signals
        .iter()
        .any(|signal| prepared_text.matches_signal(signal))
}

pub(super) fn compute_seniority_alignment(
    search_profile: &SearchProfile,
    job: &super::JobView,
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
