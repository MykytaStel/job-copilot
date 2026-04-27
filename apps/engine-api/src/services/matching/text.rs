use super::{
    EvaluatedTerms, LOW_SIGNAL_TERM_MATCH_WEIGHT, PARTIAL_PHRASE_MATCH_WEIGHT, PreparedText,
    TermSpec, is_low_signal_term, normalize_term_for_output, normalize_text, push_unique_string,
};

const REQUIRED_SKILL_WEIGHT: f32 = 1.5;
const PREFERRED_SKILL_WEIGHT: f32 = 0.7;

const REQUIRED_SECTION_MARKERS: &[&str] = &[
    "requirements",
    "required",
    "must have",
    "mandatory",
    "qualifications",
    "essential",
    "required skills",
    "must have skills",
    "minimum qualifications",
    "key requirements",
    "technical requirements",
    "minimum requirements",
    "what we require",
    "what we need",
    "you must have",
    "you should have",
    "you will need",
    "hard requirements",
    "skills required",
    "experience required",
];

const PREFERRED_SECTION_MARKERS: &[&str] = &[
    "preferred",
    "nice to have",
    "bonus",
    "desirable",
    "good to have",
    "preferred skills",
    "preferred qualifications",
    "bonus skills",
    "bonus points",
    "nice to haves",
    "would be a plus",
    "optional",
    "it would be great",
];

#[derive(Clone, Copy)]
enum SectionKind {
    Required,
    Preferred,
}

pub(super) struct SkillSections {
    required_text: Option<PreparedText>,
    preferred_text: Option<PreparedText>,
}

impl SkillSections {
    pub(super) fn skill_weight(&self, normalized: &str, significant_tokens: &[String]) -> f32 {
        let matches_in = |prepared: &PreparedText| -> bool {
            prepared.matches_signal(normalized)
                || significant_tokens
                    .iter()
                    .any(|t| prepared.matches_signal(t))
        };

        if self.required_text.as_ref().is_some_and(|t| matches_in(t)) {
            return REQUIRED_SKILL_WEIGHT;
        }
        if self.preferred_text.as_ref().is_some_and(|t| matches_in(t)) {
            return PREFERRED_SKILL_WEIGHT;
        }
        1.0
    }
}

pub(super) fn parse_skill_sections(description_text: &str) -> SkillSections {
    let mut required_lines: Vec<&str> = Vec::new();
    let mut preferred_lines: Vec<&str> = Vec::new();
    let mut current: Option<SectionKind> = None;

    for line in description_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(kind) = detect_section_header(trimmed) {
            current = Some(kind);
            continue;
        }
        match current {
            Some(SectionKind::Required) => required_lines.push(trimmed),
            Some(SectionKind::Preferred) => preferred_lines.push(trimmed),
            None => {}
        }
    }

    SkillSections {
        required_text: if required_lines.is_empty() {
            None
        } else {
            Some(PreparedText::new(&required_lines.join(" ")))
        },
        preferred_text: if preferred_lines.is_empty() {
            None
        } else {
            Some(PreparedText::new(&preferred_lines.join(" ")))
        },
    }
}

fn detect_section_header(line: &str) -> Option<SectionKind> {
    if line.len() > 80 {
        return None;
    }
    let clean: String = line
        .to_lowercase()
        .replace('-', " ")
        .chars()
        .filter(|c| c.is_alphabetic() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if REQUIRED_SECTION_MARKERS.iter().any(|&m| clean == m) {
        return Some(SectionKind::Required);
    }
    if PREFERRED_SECTION_MARKERS.iter().any(|&m| clean == m) {
        return Some(SectionKind::Preferred);
    }
    None
}

pub(super) fn evaluate_terms_section_aware(
    prepared_text: &PreparedText,
    terms: &[String],
    ignored_terms: &[String],
    sections: &SkillSections,
) -> EvaluatedTerms {
    let mut matched_terms = Vec::new();
    let mut missing_terms = Vec::new();
    let mut matched_strength = 0.0f32;
    let mut total_weight = 0.0f32;
    let mut eligible_terms = 0usize;

    for term in build_term_specs(terms) {
        if ignored_terms.contains(&term.normalized)
            || ignored_terms.contains(&term.canonical_normalized)
        {
            continue;
        }

        eligible_terms += 1;
        let importance = sections.skill_weight(&term.normalized, &term.significant_tokens);
        total_weight += importance;

        let Some((output, quality)) = match_term(prepared_text, &term) else {
            push_unique_string(&mut missing_terms, term.canonical_output.clone());
            continue;
        };

        matched_strength += quality * importance;
        push_unique_string(&mut matched_terms, output);
    }

    EvaluatedTerms {
        matched_terms,
        missing_terms,
        matched_strength,
        eligible_terms,
        total_weight,
    }
}

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
    let mut matched_strength = 0.0f32;
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
        total_weight: eligible_terms as f32,
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
