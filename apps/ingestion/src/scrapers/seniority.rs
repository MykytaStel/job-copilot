use std::sync::OnceLock;

use regex::Regex;

pub fn infer_seniority(title: &str) -> Option<String> {
    infer_seniority_from_title_and_description(title, None)
}

pub fn infer_seniority_from_title_and_description(
    title: &str,
    description: Option<&str>,
) -> Option<String> {
    infer_seniority_from_title(title)
        .or_else(|| description.and_then(infer_seniority_from_description))
}

fn infer_seniority_from_title(title: &str) -> Option<String> {
    if title_lead_re().is_match(title) {
        Some("lead".to_string())
    } else if title_junior_re().is_match(title) {
        Some("junior".to_string())
    } else if title_middle_re().is_match(title) {
        Some("middle".to_string())
    } else if title_senior_re().is_match(title) {
        Some("senior".to_string())
    } else {
        infer_seniority_from_years(title)
    }
}

fn infer_seniority_from_description(description: &str) -> Option<String> {
    if description_junior_re().is_match(description) {
        Some("junior".to_string())
    } else if description_middle_re().is_match(description) {
        Some("middle".to_string())
    } else if description_lead_re().is_match(description) {
        Some("lead".to_string())
    } else if description_senior_re().is_match(description) {
        Some("senior".to_string())
    } else {
        infer_seniority_from_years(description)
    }
}

fn infer_seniority_from_years(text: &str) -> Option<String> {
    if let Some(captures) = years_range_re().captures(text) {
        let start = captures.get(1)?.as_str().parse::<u8>().ok()?;
        let end = captures.get(2)?.as_str().parse::<u8>().ok()?;
        return seniority_from_year_range(start, end);
    }

    let captures = years_plus_re().captures(text)?;
    let years = captures.get(1)?.as_str().parse::<u8>().ok()?;
    seniority_from_years(years)
}

fn seniority_from_year_range(start: u8, end: u8) -> Option<String> {
    if start == 2 && end == 4 {
        Some("middle".to_string())
    } else {
        seniority_from_years(end)
    }
}

fn seniority_from_years(years: u8) -> Option<String> {
    match years {
        0..=1 => Some("junior".to_string()),
        2..=3 => Some("middle".to_string()),
        4..=6 => Some("senior".to_string()),
        _ => Some("lead".to_string()),
    }
}

fn title_junior_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(?:\bjunior\b|\bjr(?:\.|\b)|\bentry[-\s]?level\b|\bintern\b|\btrainee\b|\bпочатківець\b|\bмолодший\b)")
            .expect("valid junior seniority regex")
    })
}

fn title_middle_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(?:middle|mid[-\s]?level|mid|regular|intermediate|середній)\b")
            .expect("valid middle seniority regex")
    })
}

fn title_senior_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(?:\bsenior\b|\bsr(?:\.|\b)|\bдосвідчений\b)")
            .expect("valid senior seniority regex")
    })
}

fn title_lead_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(?:tech\s+lead|team\s+lead|head\s+of|principal|staff|lead)\b")
            .expect("valid lead seniority regex")
    })
}

fn description_junior_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| title_junior_re().clone())
}

fn description_middle_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| title_middle_re().clone())
}

fn description_senior_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(?:\bsenior\b|\bsr(?:\.|\b)|\bдосвідчений\b|\blead\b)")
            .expect("valid description seniority regex")
    })
}

fn description_lead_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(?:tech\s+lead|team\s+lead|head\s+of|principal|staff)\b")
            .expect("valid lead description seniority regex")
    })
}

fn years_plus_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(\d{1,2})\s*\+\s*(?:years?|yrs?|рок(?:и|ів)?)\b")
            .expect("valid plus years regex")
    })
}

fn years_range_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)\b(\d{1,2})\s*[-–]\s*(\d{1,2})\s*(?:years?|yrs?|рок(?:и|ів)?)\b")
            .expect("valid range years regex")
    })
}
