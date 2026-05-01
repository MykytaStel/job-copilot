use crate::domain::job::model::JobView;

pub const STALE_AFTER_DAYS: i64 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobAgeSignals {
    pub days_old: i64,
    pub score_delta: i16,
    pub stale: bool,
    pub source: &'static str,
}

pub fn assess_job_age(job: &JobView) -> JobAgeSignals {
    // Current schema uses jobs.last_seen_at as the last-confirmed-active timestamp.
    // If that is unavailable/unparseable, fall back to first_seen_at.
    assess_job_age_from_dates(
        Some(job.job.last_seen_at.as_str()),
        job.first_seen_at.as_str(),
        current_days_since_epoch(),
    )
}

pub fn is_stale_job_view(job: &JobView) -> bool {
    assess_job_age(job).stale
}

pub fn is_stale_from_dates(last_confirmed_active_at: Option<&str>, first_seen_at: &str) -> bool {
    assess_job_age_from_dates(
        last_confirmed_active_at,
        first_seen_at,
        current_days_since_epoch(),
    )
    .stale
}

pub fn job_age_score_delta(days_old: i64) -> i16 {
    match days_old {
        days if days >= 30 => -15,
        days if days >= 21 => -10,
        days if days >= 15 => -5,
        _ => 0,
    }
}

fn assess_job_age_from_dates(
    last_confirmed_active_at: Option<&str>,
    first_seen_at: &str,
    today_days_since_epoch: i64,
) -> JobAgeSignals {
    let (date_str, source) = last_confirmed_active_at
        .filter(|value| !value.trim().is_empty())
        .map(|value| (value, "last_confirmed_active_at"))
        .unwrap_or((first_seen_at, "first_seen_at"));

    let parsed_day = parse_days_since_epoch(date_str)
        .or_else(|| parse_days_since_epoch(first_seen_at))
        .unwrap_or(today_days_since_epoch);

    let days_old = (today_days_since_epoch - parsed_day).max(0);

    JobAgeSignals {
        days_old,
        score_delta: job_age_score_delta(days_old),
        stale: days_old >= STALE_AFTER_DAYS,
        source,
    }
}

fn parse_days_since_epoch(datetime_str: &str) -> Option<i64> {
    let s = datetime_str.trim().get(..10)?;
    let year: i64 = s[0..4].parse().ok()?;
    let month: i64 = s[5..7].parse().ok()?;
    let day: i64 = s[8..10].parse().ok()?;

    Some(civil_to_days(year, month, day))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn days(value: &str) -> i64 {
        parse_days_since_epoch(value).expect("fixture date should parse")
    }

    #[test]
    fn job_age_score_delta_covers_all_ranges() {
        assert_eq!(job_age_score_delta(0), 0);
        assert_eq!(job_age_score_delta(13), 0);
        assert_eq!(job_age_score_delta(14), 0);

        assert_eq!(job_age_score_delta(15), -5);
        assert_eq!(job_age_score_delta(20), -5);

        assert_eq!(job_age_score_delta(21), -10);
        assert_eq!(job_age_score_delta(29), -10);

        assert_eq!(job_age_score_delta(30), -15);
        assert_eq!(job_age_score_delta(60), -15);
    }

    #[test]
    fn job_age_signals_use_last_confirmed_active_before_first_seen() {
        let signal = assess_job_age_from_dates(
            Some("2026-04-20T00:00:00Z"),
            "2026-03-01T00:00:00Z",
            days("2026-04-27"),
        );

        assert_eq!(signal.days_old, 7);
        assert_eq!(signal.score_delta, 0);
        assert!(!signal.stale);
        assert_eq!(signal.source, "last_confirmed_active_at");
    }

    #[test]
    fn job_age_signals_fall_back_to_first_seen() {
        let signal = assess_job_age_from_dates(None, "2026-04-06T00:00:00Z", days("2026-04-27"));

        assert_eq!(signal.days_old, 21);
        assert_eq!(signal.score_delta, -10);
        assert!(!signal.stale);
        assert_eq!(signal.source, "first_seen_at");
    }

    #[test]
    fn job_age_signals_mark_sixty_day_jobs_as_stale() {
        let signal = assess_job_age_from_dates(
            Some("2026-02-26T00:00:00Z"),
            "2026-02-01T00:00:00Z",
            days("2026-04-27"),
        );

        assert_eq!(signal.days_old, 60);
        assert_eq!(signal.score_delta, -15);
        assert!(signal.stale);
    }
}
