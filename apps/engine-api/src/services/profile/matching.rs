const RAW_ALIAS_REPLACEMENTS: &[(&str, &str)] = &[
    ("c++", " cpp "),
    ("c#", " csharp "),
    ("node.js", " nodejs "),
    ("next.js", " nextjs "),
    ("postgresql", " postgres "),
    ("react.js", " react "),
    ("reactnative", " react native "),
];

const PHRASE_REWRITES: &[(&[&str], &str)] = &[
    (&["c", "plus", "plus"], "cpp"),
    (&["candidate", "screening"], "candidate_screening"),
    (&["distributed", "systems"], "distributed_systems"),
    (&["google", "ads"], "google_ads"),
    (&["lead", "generation"], "lead_generation"),
    (&["product", "management"], "product_management"),
    (&["project", "management"], "project_management"),
    (&["quality", "assurance"], "quality_assurance"),
    (&["social", "media"], "social_media"),
    (&["talent", "acquisition"], "talent_acquisition"),
    (&["test", "automation"], "test_automation"),
    (&["customer", "support"], "customer_support"),
    (&["data", "analyst"], "data_analyst"),
    (&["design", "system"], "design_system"),
    (&["front", "end"], "frontend"),
    (&["back", "end"], "backend"),
    (&["full", "stack"], "fullstack"),
    (&["help", "desk"], "help_desk"),
    (&["node", "js"], "nodejs"),
    (&["next", "js"], "nextjs"),
    (&["rest", "api"], "rest_api"),
    (
        &["site", "reliability", "engineer"],
        "site_reliability_engineer",
    ),
    (&["power", "bi"], "power_bi"),
    (&["product", "company"], "product_company"),
    (&["product", "manager"], "product_manager"),
    (&["project", "manager"], "project_manager"),
    (&["react", "native"], "react_native"),
    (&["c", "sharp"], "csharp"),
];

pub(crate) struct PreparedText {
    normalized_text: String,
    tokens: Vec<String>,
}

impl PreparedText {
    pub(crate) fn new(raw: &str) -> Self {
        let normalized_text = normalize_text(raw);
        let tokens = tokenize(&normalized_text);

        Self {
            normalized_text,
            tokens,
        }
    }

    pub(crate) fn matches_signal(&self, signal: &str) -> bool {
        matches_signal(&self.normalized_text, &self.tokens, signal)
    }
}

pub(crate) fn normalize_text(raw: &str) -> String {
    let mut lowered = raw.to_lowercase();

    for (needle, replacement) in RAW_ALIAS_REPLACEMENTS {
        lowered = lowered.replace(needle, replacement);
    }

    let cleaned = lowered
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>();
    let tokens = cleaned.split_whitespace().collect::<Vec<_>>();

    canonicalize_phrase_tokens(&tokens)
}

pub(crate) fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_string).collect()
}

pub(crate) fn normalize_term_for_output(raw: &str) -> String {
    normalize_text(raw).replace('_', " ")
}

pub(crate) fn contains_token(tokens: &[String], token: &str) -> bool {
    let normalized_token = normalize_text(token);

    if normalized_token.is_empty() || normalized_token.contains(' ') {
        return false;
    }

    tokens.iter().any(|existing| existing == &normalized_token)
}

pub(crate) fn contains_phrase(normalized_text: &str, phrase: &str) -> bool {
    let normalized_phrase = normalize_text(phrase);

    if normalized_phrase.is_empty() {
        return false;
    }

    let haystack = format!(" {} ", normalized_text);
    let needle = format!(" {} ", normalized_phrase);

    haystack.contains(&needle)
}

pub(crate) fn matches_signal(normalized_text: &str, tokens: &[String], signal: &str) -> bool {
    let normalized_signal = normalize_text(signal);

    if normalized_signal.is_empty() {
        return false;
    }

    if normalized_signal.contains(' ') {
        contains_phrase(normalized_text, &normalized_signal)
    } else {
        contains_token(tokens, &normalized_signal)
    }
}

fn canonicalize_phrase_tokens(tokens: &[&str]) -> String {
    let mut normalized = Vec::with_capacity(tokens.len());
    let mut index = 0usize;

    while index < tokens.len() {
        let mut matched = false;

        for (pattern, replacement) in PHRASE_REWRITES {
            if index + pattern.len() > tokens.len()
                || &tokens[index..index + pattern.len()] != *pattern
            {
                continue;
            }

            normalized.push((*replacement).to_string());
            index += pattern.len();
            matched = true;
            break;
        }

        if matched {
            continue;
        }

        normalized.push(tokens[index].to_string());
        index += 1;
    }

    normalized.join(" ")
}
