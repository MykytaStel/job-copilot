pub fn normalize_text(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

pub fn normalized_non_empty(value: Option<&str>) -> Option<String> {
    let cleaned = normalize_text(value?);
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

pub fn cleanup_description_text(
    value: &str,
    title: &str,
    company_name: &str,
    cut_markers: &[&str],
) -> String {
    let mut cleaned = normalize_text(value);

    for marker in cut_markers.iter().chain(DESCRIPTION_CUT_MARKERS.iter()) {
        if let Some(head) = truncate_at_marker(&cleaned, marker) {
            cleaned = head.trim().to_string();
        }
    }

    for prefix in [title, company_name] {
        let normalized_prefix = normalize_text(prefix);
        if !normalized_prefix.is_empty() && cleaned.starts_with(&normalized_prefix) {
            cleaned = cleaned[normalized_prefix.len()..].trim().to_string();
        }
    }

    normalize_text(&cleaned)
}

pub fn collect_text(el: &scraper::ElementRef) -> String {
    el.text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn choose_better_description(
    current: &str,
    candidate: Option<&str>,
    title: &str,
    company_name: &str,
) -> String {
    let current_clean = cleanup_description_text(current, title, company_name, &[]);
    let current_quality = score_description_quality(current, &current_clean, title, company_name);

    let Some(candidate_value) = candidate else {
        return if current_clean.is_empty() {
            title.to_string()
        } else {
            current_clean
        };
    };

    let candidate_clean = cleanup_description_text(candidate_value, title, company_name, &[]);
    if candidate_clean.is_empty() {
        return if current_clean.is_empty() {
            title.to_string()
        } else {
            current_clean
        };
    }

    let candidate_quality =
        score_description_quality(candidate_value, &candidate_clean, title, company_name);

    if current_clean.is_empty() || current_clean.eq_ignore_ascii_case(title) {
        return candidate_clean;
    }

    if candidate_quality >= current_quality + 12 {
        return candidate_clean;
    }

    if !candidate_clean.eq_ignore_ascii_case(&current_clean)
        && candidate_clean.contains(&current_clean)
    {
        return candidate_clean;
    }

    current_clean
}

fn truncate_at_marker(value: &str, marker: &str) -> Option<String> {
    let value_lower = value.to_lowercase();
    let marker_lower = marker.to_lowercase();
    let index = value_lower.find(&marker_lower)?;

    Some(value[..index].to_string())
}

fn score_description_quality(raw: &str, cleaned: &str, title: &str, company_name: &str) -> usize {
    if cleaned.is_empty() {
        return 0;
    }

    let normalized_title = normalize_text(title).to_lowercase();
    let normalized_company = normalize_text(company_name).to_lowercase();
    let useful_length = cleaned.chars().count();
    let block_count = raw
        .lines()
        .map(str::trim)
        .filter(|line| line.len() > 24)
        .count()
        .max(1);
    let sentence_count = cleaned
        .split(['.', '!', '?', ';', ':'])
        .map(str::trim)
        .filter(|segment| segment.len() > 24)
        .count();
    let unique_terms = cleaned
        .split_whitespace()
        .map(|term| {
            term.trim_matches(|ch: char| !ch.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|term| term.len() > 3)
        .filter(|term| term != &normalized_title && term != &normalized_company)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let cleaned_lower = cleaned.to_lowercase();
    let noise_hits = DESCRIPTION_CUT_MARKERS
        .iter()
        .filter(|marker| cleaned_lower.contains(&marker.to_lowercase()))
        .count();

    useful_length / 20 + block_count * 4 + sentence_count * 3 + unique_terms - noise_hits * 8
}

const DESCRIPTION_CUT_MARKERS: &[&str] = &[
    "how to apply",
    "apply now",
    "apply on company website",
    "similar vacancies",
    "related jobs",
    "схожі вакансії",
    "відгукнутися",
    "відгукнутись",
    "правила відгуків",
];
