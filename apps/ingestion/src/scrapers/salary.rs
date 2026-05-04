use std::sync::OnceLock;

use regex::Regex;

pub const EUR_TO_USD_RATE: f64 = 1.10;
pub const UAH_TO_USD_RATE: f64 = 0.024;
pub const HOURLY_TO_MONTHLY_HOURS: f64 = 160.0;
pub const ANNUAL_TO_MONTHLY_DIVISOR: f64 = 12.0;

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

pub fn normalize_salary_to_usd_monthly(
    salary_min: Option<i32>,
    salary_max: Option<i32>,
    salary_currency: Option<&str>,
    source_text: &str,
) -> (Option<i32>, Option<i32>) {
    let Some(currency) = salary_currency else {
        return (None, None);
    };

    let Some(exchange_rate) = usd_exchange_rate(currency) else {
        return (None, None);
    };
    let period_multiplier = salary_period_multiplier(source_text);

    (
        normalize_salary_amount(salary_min, exchange_rate, period_multiplier),
        normalize_salary_amount(salary_max, exchange_rate, period_multiplier),
    )
}

#[allow(clippy::type_complexity)]
pub fn parse_salary_range_with_usd_monthly(
    text: &str,
) -> (
    Option<i32>,
    Option<i32>,
    Option<String>,
    Option<i32>,
    Option<i32>,
) {
    let (salary_min, salary_max, salary_currency) = parse_salary_range(text);
    let (salary_usd_min, salary_usd_max) =
        normalize_salary_to_usd_monthly(salary_min, salary_max, salary_currency.as_deref(), text);

    (
        salary_min,
        salary_max,
        salary_currency,
        salary_usd_min,
        salary_usd_max,
    )
}

fn normalize_salary_amount(
    amount: Option<i32>,
    exchange_rate: f64,
    period_multiplier: f64,
) -> Option<i32> {
    amount.map(|value| (value as f64 * exchange_rate * period_multiplier).round() as i32)
}

fn usd_exchange_rate(currency: &str) -> Option<f64> {
    match currency.trim().to_uppercase().as_str() {
        "USD" => Some(1.0),
        "EUR" => Some(EUR_TO_USD_RATE),
        "UAH" => Some(UAH_TO_USD_RATE),
        _ => None,
    }
}

fn salary_period_multiplier(text: &str) -> f64 {
    let normalized = text.to_lowercase();

    if hourly_salary_re().is_match(&normalized) {
        HOURLY_TO_MONTHLY_HOURS
    } else if annual_salary_re().is_match(&normalized) {
        1.0 / ANNUAL_TO_MONTHLY_DIVISOR
    } else {
        1.0
    }
}

fn hourly_salary_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?iu)(?:/|\bper\s+|\bза\s+|\bна\s+)(?:hour|hr|годину|год\b)|\b(?:hourly|погодинно)\b",
        )
        .expect("valid hourly salary regex")
    })
}

fn annual_salary_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?iu)(?:/|\bper\s+|\bза\s+|\bна\s+|\bin\s+|\bв\s+)(?:year|yr|annum|рік)|\b(?:annual|annually|yearly|річна)\b")
            .expect("valid annual salary regex")
    })
}
