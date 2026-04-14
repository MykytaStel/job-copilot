use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};
use crate::domain::source::SourceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobPresentation {
    pub title: String,
    pub company: String,
    pub summary: Option<String>,
    pub location_label: Option<String>,
    pub work_mode_label: Option<String>,
    pub source_label: Option<String>,
    pub outbound_url: Option<String>,
    pub salary_label: Option<String>,
    pub freshness_label: Option<String>,
    pub badges: Vec<String>,
}

pub fn build_job_presentation(job: &Job) -> JobPresentation {
    build_presentation(job, None, None, None)
}

pub fn build_job_view_presentation(view: &JobView) -> JobPresentation {
    build_presentation(
        &view.job,
        Some(&view.first_seen_at),
        Some(&view.lifecycle_stage),
        view.primary_variant.as_ref(),
    )
}

fn build_presentation(
    job: &Job,
    first_seen_at: Option<&str>,
    lifecycle_stage: Option<&JobLifecycleStage>,
    primary_variant: Option<&JobSourceVariant>,
) -> JobPresentation {
    let source_id =
        primary_variant.and_then(|variant| SourceId::parse_canonical_key(variant.source.trim()));
    let title = normalize_label(&job.title);
    let company = normalize_label(&job.company_name);
    let location_label = build_location_label(primary_variant, &job.description_text);
    let work_mode_label = build_work_mode_label(
        job.remote_type.as_deref(),
        primary_variant,
        &job.description_text,
    );
    let source_label = source_id
        .map(|source| source.display_name().to_string())
        .or_else(|| primary_variant.map(|variant| prettify_source_label(&variant.source)));
    let outbound_url = build_outbound_url(source_id, primary_variant);
    let salary_label = build_salary_label(
        job.salary_min,
        job.salary_max,
        job.salary_currency.as_deref(),
    );
    let freshness_label =
        build_freshness_label(job.posted_at.as_deref(), first_seen_at, &job.last_seen_at);
    let summary = build_summary(
        source_id,
        &title,
        &company,
        primary_variant,
        &job.description_text,
        location_label.as_deref(),
        work_mode_label.as_deref(),
        salary_label.as_deref(),
        job.seniority.as_deref(),
    );
    let badges = build_badges(
        work_mode_label.as_deref(),
        job.seniority.as_deref(),
        lifecycle_stage,
    );

    JobPresentation {
        title,
        company,
        summary,
        location_label,
        work_mode_label,
        source_label,
        outbound_url,
        salary_label,
        freshness_label,
        badges,
    }
}

fn build_summary(
    source_id: Option<SourceId>,
    title: &str,
    company: &str,
    primary_variant: Option<&JobSourceVariant>,
    description_text: &str,
    location_label: Option<&str>,
    work_mode_label: Option<&str>,
    salary_label: Option<&str>,
    seniority: Option<&str>,
) -> Option<String> {
    let raw_description = raw_string(primary_variant, "description_text")
        .unwrap_or_else(|| description_text.to_string());
    let cleaned_description = clean_summary_text(&raw_description);

    if let Some(summary) =
        extract_description_summary(source_id, &cleaned_description, title, company)
    {
        return Some(summary);
    }

    build_metadata_summary(location_label, work_mode_label, salary_label, seniority)
}

fn build_metadata_summary(
    location_label: Option<&str>,
    work_mode_label: Option<&str>,
    salary_label: Option<&str>,
    seniority: Option<&str>,
) -> Option<String> {
    let seniority = seniority
        .map(normalize_label)
        .filter(|value| !value.is_empty());
    let role_prefix = match (seniority.as_deref(), work_mode_label) {
        (Some(level), Some(mode)) => Some(format!("{level} {mode} role")),
        (Some(level), None) => Some(format!("{level} role")),
        (None, Some(mode)) => Some(format!("{mode} role")),
        (None, None) => None,
    };

    let mut parts = Vec::new();

    if let Some(prefix) = role_prefix {
        parts.push(prefix);
    }

    if let Some(location) = location_label {
        parts.push(format!("Location: {location}"));
    }

    if let Some(salary) = salary_label {
        parts.push(format!("Salary: {salary}"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(". "))
    }
}

fn extract_description_summary(
    source_id: Option<SourceId>,
    description: &str,
    title: &str,
    company: &str,
) -> Option<String> {
    if description.is_empty() {
        return None;
    }

    let title_normalized = normalized_cmp(title);
    let company_normalized = normalized_cmp(company);

    let primary = trim_summary(description);

    if is_meaningful_summary(&primary, &title_normalized, &company_normalized, source_id) {
        return Some(primary);
    }

    for candidate in description
        .split(['\n', '.', ';'])
        .map(trim_summary)
        .filter(|value| !value.is_empty())
    {
        if is_meaningful_summary(
            &candidate,
            &title_normalized,
            &company_normalized,
            source_id,
        ) {
            return Some(candidate);
        }
    }

    None
}

fn is_meaningful_summary(
    value: &str,
    title_normalized: &str,
    company_normalized: &str,
    source_id: Option<SourceId>,
) -> bool {
    let normalized = normalized_cmp(value);

    if normalized.len() < 24 {
        return false;
    }

    if normalized == *title_normalized || normalized == *company_normalized {
        return false;
    }

    let word_count = value.split_whitespace().count();
    if word_count < 4 {
        return false;
    }

    match source_id {
        Some(SourceId::Djinni) => {
            !normalized.contains("view vacancy")
                && !normalized.contains("send cv")
                && !normalized.starts_with("salary")
        }
        Some(SourceId::WorkUa) => {
            !normalized.starts_with("company") && !normalized.starts_with("save vacancy")
        }
        Some(SourceId::RobotaUa) => {
            !normalized.starts_with("vidguknutis") && !normalized.starts_with("відгукнутись")
        }
        None => true,
    }
}

fn build_location_label(
    primary_variant: Option<&JobSourceVariant>,
    _description_text: &str,
) -> Option<String> {
    let location = raw_string(primary_variant, "location")?;
    let cleaned = clean_location_label(&location);

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn build_work_mode_label(
    remote_type: Option<&str>,
    primary_variant: Option<&JobSourceVariant>,
    description_text: &str,
) -> Option<String> {
    let raw_location = raw_string(primary_variant, "location");
    let raw_remote_type = raw_string(primary_variant, "remote_type");
    let source_text = raw_remote_type
        .as_deref()
        .or(remote_type)
        .or(raw_location.as_deref())
        .unwrap_or(description_text);

    normalize_work_mode(source_text)
}

fn build_outbound_url(
    source_id: Option<SourceId>,
    primary_variant: Option<&JobSourceVariant>,
) -> Option<String> {
    let variant = primary_variant?;
    let source_job_id = variant.source_job_id.trim();
    let source_url = sanitize_https_url(variant.source_url.trim());

    match source_id {
        Some(SourceId::RobotaUa) if !source_job_id.is_empty() => {
            Some(format!("https://robota.ua/vacancy/{source_job_id}"))
        }
        Some(SourceId::WorkUa) if !source_job_id.is_empty() => {
            Some(format!("https://www.work.ua/jobs/{source_job_id}/"))
        }
        Some(SourceId::Djinni) => source_url.filter(|value| value.contains("://djinni.co/jobs/")),
        Some(SourceId::WorkUa) => source_url.filter(|value| value.contains("://www.work.ua/")),
        Some(SourceId::RobotaUa) => source_url.filter(|value| value.contains("://robota.ua/")),
        None => source_url,
    }
}

fn build_salary_label(
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    salary_currency: Option<&str>,
) -> Option<String> {
    let currency = salary_currency
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("salary");

    match (salary_min, salary_max) {
        (Some(min), Some(max)) if min > 0 && max > 0 => Some(format!(
            "{}-{} {currency}",
            format_number(min),
            format_number(max)
        )),
        (Some(min), _) if min > 0 => Some(format!("from {} {currency}", format_number(min))),
        (_, Some(max)) if max > 0 => Some(format!("up to {} {currency}", format_number(max))),
        _ => None,
    }
}

fn build_freshness_label(
    posted_at: Option<&str>,
    first_seen_at: Option<&str>,
    last_seen_at: &str,
) -> Option<String> {
    if let Some(posted) = date_part(posted_at) {
        return Some(format!("Posted {posted}"));
    }

    if let Some(first_seen) = date_part(first_seen_at) {
        return Some(format!("Seen {first_seen}"));
    }

    date_part(Some(last_seen_at)).map(|date| format!("Seen {date}"))
}

fn build_badges(
    work_mode_label: Option<&str>,
    seniority: Option<&str>,
    lifecycle_stage: Option<&JobLifecycleStage>,
) -> Vec<String> {
    let mut badges = Vec::new();

    if let Some(mode) = work_mode_label {
        push_badge(&mut badges, mode.to_string());
    }

    if let Some(level) = seniority
        .map(normalize_label)
        .filter(|value| !value.is_empty())
    {
        push_badge(&mut badges, level);
    }

    match lifecycle_stage {
        Some(JobLifecycleStage::Reactivated) => push_badge(&mut badges, "Reactivated".to_string()),
        Some(JobLifecycleStage::Inactive) => push_badge(&mut badges, "Inactive".to_string()),
        _ => {}
    }

    badges
}

fn raw_string(primary_variant: Option<&JobSourceVariant>, key: &str) -> Option<String> {
    primary_variant?
        .raw_payload
        .as_ref()?
        .get(key)?
        .as_str()
        .map(normalize_label)
        .filter(|value| !value.is_empty())
}

fn normalize_work_mode(value: &str) -> Option<String> {
    let normalized = normalized_cmp(value);

    if normalized.is_empty() {
        return None;
    }

    if normalized.contains("remote") || normalized.contains("віддал") {
        return Some("Remote".to_string());
    }

    if normalized.contains("hybrid") || normalized.contains("гібрид") {
        return Some("Hybrid".to_string());
    }

    if normalized.contains("office")
        || normalized.contains("onsite")
        || normalized.contains("on site")
        || normalized.contains("офіс")
    {
        return Some("On-site".to_string());
    }

    None
}

fn clean_location_label(value: &str) -> String {
    let mut cleaned = normalize_label(value);

    for token in [
        "Віддалена робота",
        "Віддалено",
        "Remote work",
        "Remote",
        "Hybrid",
        "On-site",
    ] {
        cleaned = cleaned.replace(token, "");
    }

    normalize_label(cleaned.trim_matches(|c: char| matches!(c, ',' | '|' | '-' | '/')))
}

fn clean_summary_text(value: &str) -> String {
    let mut cleaned = normalize_label(value);

    for (from, to) in [
        ("&nbsp;", " "),
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
    ] {
        cleaned = cleaned.replace(from, to);
    }

    cleaned
}

fn trim_summary(value: &str) -> String {
    truncate_with_ellipsis(normalize_label(value).trim_matches('.').to_string(), 220)
}

fn truncate_with_ellipsis(value: String, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value;
    }

    let mut truncated = String::new();

    for ch in value.chars().take(max_len.saturating_sub(3)) {
        truncated.push(ch);
    }

    truncated.push_str("...");
    truncated
}

fn normalize_label(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalized_cmp(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_https_url(value: &str) -> Option<String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix("https://") {
        return Some(format!("https://{}", rest.trim_end_matches('/')));
    }

    if let Some(rest) = trimmed.strip_prefix("http://") {
        return Some(format!("https://{}", rest.trim_end_matches('/')));
    }

    None
}

fn prettify_source_label(source: &str) -> String {
    source
        .split('_')
        .filter(|part| !part.is_empty())
        .map(capitalize_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize_word(value: &str) -> String {
    let mut chars = value.chars();

    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn format_number(value: i32) -> String {
    let digits = value.abs().to_string();
    let mut reversed = String::new();

    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            reversed.push(',');
        }
        reversed.push(ch);
    }

    let formatted = reversed.chars().rev().collect::<String>();

    if value < 0 {
        format!("-{formatted}")
    } else {
        formatted
    }
}

fn date_part(value: Option<&str>) -> Option<String> {
    value.and_then(|value| value.get(..10)).map(str::to_string)
}

fn push_badge(target: &mut Vec<String>, value: String) {
    if !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};

    use super::build_job_view_presentation;

    fn sample_view(
        source: &str,
        source_job_id: &str,
        source_url: &str,
        description_text: &str,
        raw_payload: serde_json::Value,
    ) -> JobView {
        JobView {
            job: Job {
                id: format!("job-{source_job_id}"),
                title: "Senior Backend Engineer".to_string(),
                company_name: "SignalHire".to_string(),
                remote_type: Some("remote".to_string()),
                seniority: Some("senior".to_string()),
                description_text: description_text.to_string(),
                salary_min: Some(5000),
                salary_max: Some(6500),
                salary_currency: Some("USD".to_string()),
                posted_at: Some("2026-04-12T08:00:00Z".to_string()),
                last_seen_at: "2026-04-14T10:00:00Z".to_string(),
                is_active: true,
            },
            first_seen_at: "2026-04-12T08:00:00Z".to_string(),
            inactivated_at: None,
            reactivated_at: None,
            lifecycle_stage: JobLifecycleStage::Active,
            primary_variant: Some(JobSourceVariant {
                source: source.to_string(),
                source_job_id: source_job_id.to_string(),
                source_url: source_url.to_string(),
                raw_payload: Some(raw_payload),
                fetched_at: "2026-04-14T10:00:00Z".to_string(),
                last_seen_at: "2026-04-14T10:00:00Z".to_string(),
                is_active: true,
                inactivated_at: None,
            }),
        }
    }

    #[test]
    fn djinni_normalization_returns_stable_presentation_fields() {
        let view = sample_view(
            "djinni",
            "196044",
            "https://djinni.co/jobs/196044-seo-specialist/",
            "   Build Rust APIs for high-load recruiting workflows.\nRemote team with async collaboration.   ",
            json!({
                "location": "Remote, Europe",
                "description_text": "Build Rust APIs for high-load recruiting workflows. Remote team with async collaboration."
            }),
        );

        let presentation = build_job_view_presentation(&view);

        assert_eq!(presentation.source_label.as_deref(), Some("Djinni"));
        assert_eq!(
            presentation.outbound_url.as_deref(),
            Some("https://djinni.co/jobs/196044-seo-specialist")
        );
        assert_eq!(
            presentation.summary.as_deref(),
            Some(
                "Build Rust APIs for high-load recruiting workflows. Remote team with async collaboration"
            )
        );
        assert_eq!(presentation.location_label.as_deref(), Some("Europe"));
        assert_eq!(presentation.work_mode_label.as_deref(), Some("Remote"));
        assert_eq!(
            presentation.salary_label.as_deref(),
            Some("5,000-6,500 USD")
        );
        assert_eq!(
            presentation.freshness_label.as_deref(),
            Some("Posted 2026-04-12")
        );
    }

    #[test]
    fn robota_outbound_url_is_built_from_source_job_id() {
        let view = sample_view(
            "robota_ua",
            "10677040",
            "https://robota.ua/company6575304/vacancy10677040",
            "Lead product direction for a B2B SaaS team.",
            json!({
                "location": "Київ",
                "description_text": "Lead product direction for a B2B SaaS team."
            }),
        );

        let presentation = build_job_view_presentation(&view);

        assert_eq!(
            presentation.outbound_url.as_deref(),
            Some("https://robota.ua/vacancy/10677040")
        );
        assert_eq!(presentation.source_label.as_deref(), Some("Robota.ua"));
    }

    #[test]
    fn missing_source_url_falls_back_safely() {
        let view = sample_view(
            "work_ua",
            "87654321",
            "",
            "Own integrations with ATS partners.",
            json!({
                "location": "Kyiv",
                "description_text": "Own integrations with ATS partners."
            }),
        );

        let presentation = build_job_view_presentation(&view);

        assert_eq!(
            presentation.outbound_url.as_deref(),
            Some("https://www.work.ua/jobs/87654321/")
        );
    }

    #[test]
    fn supported_source_normalization_is_deterministic() {
        let djinni = sample_view(
            "djinni",
            "196044",
            "https://djinni.co/jobs/196044-seo-specialist/",
            "Build Rust APIs for high-load recruiting workflows.",
            json!({
                "location": "Remote, Europe",
                "description_text": "Build Rust APIs for high-load recruiting workflows."
            }),
        );
        let work = sample_view(
            "work_ua",
            "87654321",
            "https://www.work.ua/jobs/87654321/",
            "Improve the hiring funnel with product analytics.",
            json!({
                "location": "Lviv",
                "description_text": "Improve the hiring funnel with product analytics."
            }),
        );
        let robota = sample_view(
            "robota_ua",
            "10677040",
            "https://robota.ua/company6575304/vacancy10677040",
            "Own delivery for outbound automation products.",
            json!({
                "location": "Київ",
                "description_text": "Own delivery for outbound automation products."
            }),
        );

        let first = [
            build_job_view_presentation(&djinni),
            build_job_view_presentation(&work),
            build_job_view_presentation(&robota),
        ];
        let second = [
            build_job_view_presentation(&djinni),
            build_job_view_presentation(&work),
            build_job_view_presentation(&robota),
        ];

        assert_eq!(first, second);
    }
}
