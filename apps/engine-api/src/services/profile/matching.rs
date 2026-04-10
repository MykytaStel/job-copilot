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
    raw.chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_string).collect()
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
