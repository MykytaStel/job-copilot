use serde_json::json;

use crate::domain::job::model::{Job, JobLifecycleStage, JobSourceVariant, JobView};

use super::{assess_description_quality, build_job_view_presentation, JobTextQuality};

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
            location: Some("Remote, Europe".to_string()),
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
        Some("Build Rust APIs for high-load recruiting workflows. Remote team with async collaboration")
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
    assert_eq!(
        presentation.lifecycle_primary_label.as_deref(),
        Some("Posted 2026-04-12")
    );
    assert_eq!(
        presentation.lifecycle_secondary_label.as_deref(),
        Some("Last confirmed active 2026-04-14")
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
fn dou_outbound_url_uses_source_url() {
    let view = sample_view(
        "dou_ua",
        "354587",
        "https://jobs.dou.ua/companies/getcode/vacancies/354587/",
        "Працюємо над CRM/ERP-продуктом, який розвиваємо з нуля.",
        json!({
            "location": "віддалено",
            "description_text": "Працюємо над CRM/ERP-продуктом, який розвиваємо з нуля."
        }),
    );

    let presentation = build_job_view_presentation(&view);

    assert_eq!(presentation.source_label.as_deref(), Some("DOU"));
    assert_eq!(
        presentation.outbound_url.as_deref(),
        Some("https://jobs.dou.ua/companies/getcode/vacancies/354587")
    );
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
fn weak_summaries_fall_back_to_metadata_and_mark_quality() {
    let view = sample_view(
        "work_ua",
        "87654321",
        "https://www.work.ua/jobs/87654321/",
        "React team",
        json!({
            "location": "Remote",
            "description_text": "React team"
        }),
    );

    let presentation = build_job_view_presentation(&view);

    assert!(presentation.summary_fallback);
    assert_eq!(presentation.summary_quality, Some(JobTextQuality::Weak));
    assert_eq!(presentation.description_quality, JobTextQuality::Weak);
    assert!(presentation.summary.is_some());
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
    let dou = sample_view(
        "dou_ua",
        "354587",
        "https://jobs.dou.ua/companies/getcode/vacancies/354587/",
        "Працюємо над CRM/ERP-продуктом, який розвиваємо з нуля.",
        json!({
            "location": "віддалено",
            "description_text": "Працюємо над CRM/ERP-продуктом, який розвиваємо з нуля."
        }),
    );

    let first = [
        build_job_view_presentation(&djinni),
        build_job_view_presentation(&dou),
        build_job_view_presentation(&work),
        build_job_view_presentation(&robota),
    ];
    let second = [
        build_job_view_presentation(&djinni),
        build_job_view_presentation(&dou),
        build_job_view_presentation(&work),
        build_job_view_presentation(&robota),
    ];

    assert_eq!(first, second);
}

#[test]
fn missing_posted_at_uses_seen_since_for_primary_lifecycle_label() {
    let mut view = sample_view(
        "work_ua",
        "87654321",
        "https://www.work.ua/jobs/87654321/",
        "Own integrations with ATS partners.",
        json!({
            "location": "Kyiv",
            "description_text": "Own integrations with ATS partners."
        }),
    );
    view.job.posted_at = None;
    view.first_seen_at = "2026-04-15T08:00:00Z".to_string();
    view.job.last_seen_at = "2026-04-22T09:00:00Z".to_string();

    let presentation = build_job_view_presentation(&view);

    assert_eq!(
        presentation.lifecycle_primary_label.as_deref(),
        Some("Seen since 2026-04-15")
    );
    assert_eq!(
        presentation.lifecycle_secondary_label.as_deref(),
        Some("Last confirmed active 2026-04-22")
    );
}

#[test]
fn inactive_lifecycle_prefers_inactive_since_secondary_label() {
    let mut view = sample_view(
        "djinni",
        "196044",
        "https://djinni.co/jobs/196044-seo-specialist/",
        "Build Rust APIs for high-load recruiting workflows.",
        json!({
            "location": "Remote, Europe",
            "description_text": "Build Rust APIs for high-load recruiting workflows."
        }),
    );
    view.lifecycle_stage = JobLifecycleStage::Inactive;
    view.job.is_active = false;
    view.inactivated_at = Some("2026-04-20T09:00:00Z".to_string());
    view.job.last_seen_at = "2026-04-20T09:00:00Z".to_string();

    let presentation = build_job_view_presentation(&view);

    assert_eq!(
        presentation.lifecycle_primary_label.as_deref(),
        Some("Posted 2026-04-12")
    );
    assert_eq!(
        presentation.lifecycle_secondary_label.as_deref(),
        Some("Inactive since 2026-04-20")
    );
}

// --- assess_description_quality ---

#[test]
fn description_quality_is_weak_for_empty_input() {
    assert_eq!(assess_description_quality(""), JobTextQuality::Weak);
}

#[test]
fn description_quality_is_weak_for_short_text() {
    assert_eq!(assess_description_quality("Rust developer"), JobTextQuality::Weak);
}

#[test]
fn description_quality_is_strong_for_long_clean_text() {
    let long_text = "We are building a distributed data platform used by thousands of developers. \
        You will design and implement backend services in Rust, work closely with the infrastructure team, \
        and contribute to our open-source tooling. \
        The role requires deep knowledge of async Rust, Postgres, and distributed systems. \
        You will own features end-to-end, from design through production monitoring.";
    assert_eq!(assess_description_quality(long_text), JobTextQuality::Strong);
}

#[test]
fn description_quality_is_mixed_for_medium_length_text() {
    let medium_text = "Join our team to build reliable backend services. \
        We value clean code, code review discipline, and a good engineering culture. \
        Experience with Rust or Go is a strong plus.";
    assert_eq!(assess_description_quality(medium_text), JobTextQuality::Mixed);
}

#[test]
fn description_quality_is_weak_when_two_noise_markers_present() {
    let noisy = "Senior Rust engineer wanted. Apply now. How to apply: send cv to hr@example.com. \
        We also have similar vacancies on our website.";
    assert_eq!(assess_description_quality(noisy), JobTextQuality::Weak);
}

// --- build_salary_label (tested via presentation) ---

#[test]
fn salary_label_shows_range_when_both_present() {
    let view = sample_view(
        "djinni",
        "100",
        "https://djinni.co/jobs/100/",
        "Build Rust APIs for high-load recruiting workflows. Remote team with async collaboration.",
        json!({"location": "Remote", "description_text": "Build Rust APIs."}),
    );
    // sample_view already sets salary_min=5000, salary_max=6500, currency=USD
    let presentation = build_job_view_presentation(&view);
    assert_eq!(presentation.salary_label.as_deref(), Some("5,000-6,500 USD"));
}

#[test]
fn salary_label_shows_from_when_only_min() {
    let mut view = sample_view(
        "djinni",
        "101",
        "https://djinni.co/jobs/101/",
        "Build distributed backend services in Rust at scale.",
        json!({}),
    );
    view.job.salary_max = None;
    view.job.salary_currency = Some("USD".to_string());

    let presentation = build_job_view_presentation(&view);
    assert_eq!(presentation.salary_label.as_deref(), Some("from 5,000 USD"));
}

#[test]
fn salary_label_shows_up_to_when_only_max() {
    let mut view = sample_view(
        "djinni",
        "102",
        "https://djinni.co/jobs/102/",
        "Build distributed backend services in Rust at scale.",
        json!({}),
    );
    view.job.salary_min = None;
    view.job.salary_currency = Some("USD".to_string());

    let presentation = build_job_view_presentation(&view);
    assert_eq!(presentation.salary_label.as_deref(), Some("up to 6,500 USD"));
}

#[test]
fn salary_label_is_none_when_both_missing() {
    let mut view = sample_view(
        "djinni",
        "103",
        "https://djinni.co/jobs/103/",
        "Build distributed backend services in Rust at scale.",
        json!({}),
    );
    view.job.salary_min = None;
    view.job.salary_max = None;

    let presentation = build_job_view_presentation(&view);
    assert!(presentation.salary_label.is_none());
}

// --- build_lifecycle_labels for Reactivated stage ---

#[test]
fn reactivated_lifecycle_shows_reactivated_primary_and_last_confirmed_secondary() {
    let mut view = sample_view(
        "djinni",
        "200",
        "https://djinni.co/jobs/200/",
        "Build Rust APIs for high-load recruiting workflows. Remote team async.",
        json!({"location": "Remote", "description_text": "Build Rust APIs."}),
    );
    view.lifecycle_stage = JobLifecycleStage::Reactivated;
    view.reactivated_at = Some("2026-04-20T09:00:00Z".to_string());
    view.job.last_seen_at = "2026-04-22T12:00:00Z".to_string();

    let presentation = build_job_view_presentation(&view);

    assert_eq!(
        presentation.lifecycle_primary_label.as_deref(),
        Some("Reactivated 2026-04-20")
    );
    assert_eq!(
        presentation.lifecycle_secondary_label.as_deref(),
        Some("Last confirmed active 2026-04-22")
    );
}

// --- build_work_mode_label ---

#[test]
fn work_mode_label_is_hybrid_when_raw_payload_says_hybrid() {
    let mut view = sample_view(
        "work_ua",
        "300",
        "https://www.work.ua/jobs/300/",
        "Build backend services in a hybrid work environment. Postgres and Rust required.",
        json!({"remote_type": "hybrid", "location": "Kyiv"}),
    );
    // Clear the job-level remote_type so the raw payload field takes precedence.
    view.job.remote_type = None;

    let presentation = build_job_view_presentation(&view);
    assert_eq!(presentation.work_mode_label.as_deref(), Some("Hybrid"));
}

#[test]
fn work_mode_label_is_onsite_when_office_in_raw_payload() {
    let mut view = sample_view(
        "work_ua",
        "301",
        "https://www.work.ua/jobs/301/",
        "Build backend services at our Kyiv office. Strong Rust skills required for this role.",
        json!({"remote_type": "office", "location": "Kyiv"}),
    );
    // Clear the job-level remote_type so the raw payload field takes precedence.
    view.job.remote_type = None;

    let presentation = build_job_view_presentation(&view);
    assert_eq!(presentation.work_mode_label.as_deref(), Some("On-site"));
}

#[test]
fn work_mode_label_is_none_when_no_mode_signal() {
    let mut view = sample_view(
        "work_ua",
        "302",
        "https://www.work.ua/jobs/302/",
        "Build backend services. Strong Rust skills required.",
        json!({}),
    );
    view.job.remote_type = None;

    let presentation = build_job_view_presentation(&view);
    assert!(presentation.work_mode_label.is_none());
}
