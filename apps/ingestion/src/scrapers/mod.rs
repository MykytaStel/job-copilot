pub mod djinni;
pub mod robota_ua;
pub mod work_ua;

use std::time::Duration;

use tokio::time::sleep;

pub struct ScraperConfig {
    pub pages: u32,
    pub keyword: Option<String>,
    pub page_delay_ms: u64,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            pages: 3,
            keyword: None,
            page_delay_ms: 600,
        }
    }
}

pub async fn polite_delay(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

/// Parse a salary string into (min, max, currency).
/// Handles Ukrainian notation ("40 000 грн"), USD ("$3000-5000"), EUR ("€2000").
pub fn parse_salary_range(text: &str) -> (Option<i32>, Option<i32>, Option<String>) {
    let currency = if text.contains('$') || text.to_lowercase().contains("usd") {
        Some("USD".to_string())
    } else if text.contains('€') || text.to_lowercase().contains("eur") {
        Some("EUR".to_string())
    } else if text.contains("грн") || text.to_lowercase().contains("uah") {
        Some("UAH".to_string())
    } else {
        None
    };

    // Strip all whitespace so "40 000" becomes "40000", then extract digit runs.
    let stripped: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let numbers: Vec<i32> = stripped
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| s.len() >= 3) // at least 3 digits → ≥100
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();

    match numbers.as_slice() {
        [min, max, ..] => (Some(*min), Some(*max), currency),
        [single] => (Some(*single), None, currency),
        [] => (None, None, currency),
    }
}

pub fn infer_seniority(title: &str) -> Option<String> {
    let t = title.to_lowercase();
    if t.contains("junior") || t.contains("jr.") || t.contains("intern") || t.contains("trainee") {
        Some("junior".to_string())
    } else if t.contains("middle") || t.contains(" mid ") || t.contains("mid-level") {
        Some("middle".to_string())
    } else if t.contains("senior") || t.contains("sr.") {
        Some("senior".to_string())
    } else if t.contains("staff") || t.contains("principal") {
        Some("senior".to_string())
    } else if t.contains(" lead") || t.starts_with("lead ") || t.contains("tech lead") {
        Some("lead".to_string())
    } else {
        None
    }
}

pub fn infer_remote_type(text: &str) -> Option<String> {
    let t = text.to_lowercase();
    if t.contains("remote") || t.contains("remotely") || t.contains("дистанційно") || t.contains("віддален") {
        Some("remote".to_string())
    } else if t.contains("hybrid") || t.contains("гібрид") || t.contains("частково") {
        Some("hybrid".to_string())
    } else if t.contains(" office") || t.contains("в офіс") || t.contains("на місці") {
        Some("office".to_string())
    } else {
        None
    }
}

pub fn collect_text(el: &scraper::ElementRef) -> String {
    el.text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
