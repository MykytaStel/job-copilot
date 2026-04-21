use super::{
    EvaluatedTerms, LOW_SIGNAL_TERM_MATCH_WEIGHT, PARTIAL_PHRASE_MATCH_WEIGHT, PreparedText,
    TermSpec, is_low_signal_term, normalize_term_for_output, normalize_text, push_unique_string,
};

pub(super) fn build_searchable_text(job: &super::JobView) -> String {
    build_searchable_text_parts(
        &job.job,
        job.primary_variant
            .as_ref()
            .map(|variant| variant.source.as_str()),
    )
}

pub(super) fn build_searchable_text_parts(job: &super::Job, source: Option<&str>) -> String {
    let mut parts = vec![
        job.title.as_str(),
        job.company_name.as_str(),
        job.description_text.as_str(),
    ];

    if let Some(remote_type) = job.remote_type.as_deref() {
        parts.push(remote_type);
    }

    if let Some(source) = source {
        parts.push(source);
    }

    parts.join(" ")
}

pub(super) fn evaluate_terms(
    prepared_text: &PreparedText,
    terms: &[String],
    ignored_terms: &[String],
) -> EvaluatedTerms {
    let mut matched_terms = Vec::new();
    let mut missing_terms = Vec::new();
    let mut matched_strength = 0.0;
    let mut eligible_terms = 0usize;

    for term in build_term_specs(terms) {
        if ignored_terms.contains(&term.normalized)
            || ignored_terms.contains(&term.canonical_normalized)
        {
            continue;
        }

        eligible_terms += 1;

        let Some((output, strength)) = match_term(prepared_text, &term) else {
            push_unique_string(&mut missing_terms, term.canonical_output.clone());
            continue;
        };

        matched_strength += strength;
        push_unique_string(&mut matched_terms, output);
    }

    EvaluatedTerms {
        matched_terms,
        missing_terms,
        matched_strength,
        eligible_terms,
    }
}

pub(super) fn collect_missing_signals(
    matched_profile_skills: &EvaluatedTerms,
    matched_profile_keywords: &EvaluatedTerms,
    matched_search_terms: &EvaluatedTerms,
) -> Vec<String> {
    let mut missing = Vec::new();

    for term in &matched_profile_skills.missing_terms {
        push_unique_string(&mut missing, term.clone());
    }

    for term in &matched_search_terms.missing_terms {
        push_unique_string(&mut missing, term.clone());
    }

    for term in &matched_profile_keywords.missing_terms {
        push_unique_string(&mut missing, term.clone());
    }

    missing.truncate(8);
    missing
}

pub(super) fn build_term_specs(terms: &[String]) -> Vec<TermSpec> {
    let mut specs = Vec::new();

    for term in terms {
        let normalized = normalize_text(term);
        if normalized.is_empty() {
            continue;
        }

        let significant_tokens = extract_significant_tokens(&normalized);
        if significant_tokens.is_empty() {
            continue;
        }

        let canonical_normalized = significant_tokens.join(" ");
        if specs
            .iter()
            .any(|spec: &TermSpec| spec.canonical_normalized == canonical_normalized)
        {
            continue;
        }

        specs.push(TermSpec {
            output: normalize_term_for_output(term),
            canonical_output: canonical_normalized.replace('_', " "),
            normalized,
            canonical_normalized,
            significant_tokens,
        });
    }

    specs
}

pub(super) fn extract_significant_tokens(normalized: &str) -> Vec<String> {
    normalized
        .split_whitespace()
        .filter(|token| !is_low_signal_term(token))
        .map(str::to_string)
        .collect()
}

fn match_term(prepared_text: &PreparedText, term: &TermSpec) -> Option<(String, f32)> {
    if prepared_text.matches_signal(&term.normalized) {
        return Some((term.output.clone(), 1.0));
    }

    if term.significant_tokens.len() == 1 {
        let token = &term.significant_tokens[0];
        if prepared_text.matches_signal(token) {
            let output = if term.normalized == term.canonical_normalized {
                term.output.clone()
            } else {
                term.canonical_output.clone()
            };
            let weight = if term.normalized == term.canonical_normalized {
                1.0
            } else {
                LOW_SIGNAL_TERM_MATCH_WEIGHT
            };
            return Some((output, weight));
        }

        return None;
    }

    if term
        .significant_tokens
        .iter()
        .all(|token| prepared_text.matches_signal(token))
    {
        return Some((term.canonical_output.clone(), PARTIAL_PHRASE_MATCH_WEIGHT));
    }

    None
}

pub(super) fn merge_terms(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = Vec::new();

    for term in left {
        push_unique_string(&mut merged, term.clone());
    }

    for term in right {
        push_unique_string(&mut merged, term.clone());
    }

    merged
}
