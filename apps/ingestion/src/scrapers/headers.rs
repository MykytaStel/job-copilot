use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub fn build_default_headers(
    accept: &str,
    accept_language: &str,
    extra_headers: &[(&str, &str)],
) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    insert_header(&mut headers, "Accept", accept)?;
    insert_header(&mut headers, "Accept-Language", accept_language)?;

    for (name, value) in extra_headers {
        insert_header(&mut headers, name, value)?;
    }

    Ok(headers)
}

fn insert_header(headers: &mut HeaderMap, name: &str, value: &str) -> Result<(), String> {
    let header_name = HeaderName::from_bytes(name.as_bytes())
        .map_err(|error| format!("invalid header name '{name}': {error}"))?;
    let header_value = HeaderValue::from_str(value)
        .map_err(|error| format!("invalid header value for '{name}': {error}"))?;
    headers.insert(header_name, header_value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_default_headers;

    #[test]
    fn builds_headers_with_accept_language_and_extras() {
        let headers = build_default_headers(
            "text/html",
            "uk-UA,uk;q=0.9",
            &[
                ("Origin", "https://example.com"),
                ("Referer", "https://example.com/"),
            ],
        )
        .expect("headers should build");

        assert_eq!(
            headers.get("Accept").and_then(|value| value.to_str().ok()),
            Some("text/html")
        );
        assert_eq!(
            headers
                .get("Accept-Language")
                .and_then(|value| value.to_str().ok()),
            Some("uk-UA,uk;q=0.9")
        );
        assert_eq!(
            headers.get("Origin").and_then(|value| value.to_str().ok()),
            Some("https://example.com")
        );
        assert_eq!(
            headers.get("Referer").and_then(|value| value.to_str().ok()),
            Some("https://example.com/")
        );
    }
}
