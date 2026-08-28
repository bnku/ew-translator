use super::settings::TARGET_LANG;

use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::{error::Error, fmt, time::Duration};

const GOOGLE_TRANSLATE_URL: &str = "https://translate.googleapis.com/translate_a/single";

#[derive(Debug)]
pub enum TranslateError {
    EmptyPhrase,
    Request(reqwest::Error),
    HttpStatus { status: StatusCode, body: String },
    InvalidJson(serde_json::Error),
    MissingTranslation,
}

impl fmt::Display for TranslateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPhrase => write!(formatter, "No text is selected"),
            Self::Request(error) => write!(formatter, "Google Translate request failed: {error}"),
            Self::HttpStatus { status, body } => {
                write!(formatter, "Google Translate returned HTTP {status}")?;
                if !body.is_empty() {
                    write!(formatter, ": {body}")?;
                }
                Ok(())
            }
            Self::InvalidJson(error) => {
                write!(formatter, "Google Translate returned invalid JSON: {error}")
            }
            Self::MissingTranslation => {
                write!(
                    formatter,
                    "Google Translate response contains no translation"
                )
            }
        }
    }
}

impl Error for TranslateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Request(error) => Some(error),
            Self::InvalidJson(error) => Some(error),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for TranslateError {
    fn from(error: reqwest::Error) -> Self {
        Self::Request(error)
    }
}

impl From<serde_json::Error> for TranslateError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidJson(error)
    }
}

#[tokio::main]
pub async fn google(phrase: String) -> Result<String, TranslateError> {
    if phrase.trim().is_empty() {
        return Err(TranslateError::EmptyPhrase);
    }

    let target_lang = TARGET_LANG.read().unwrap().clone();
    let client = Client::builder()
        .user_agent(concat!("ew-translator/", env!("CARGO_PKG_VERSION")))
        .https_only(true)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .build()?;

    let response = client
        .get(GOOGLE_TRANSLATE_URL)
        .query(&[
            ("client", "gtx"),
            ("sl", "auto"),
            ("tl", target_lang.as_str()),
            ("dt", "t"),
            ("q", phrase.as_str()),
        ])
        .send()
        .await?;
    let status = response.status();
    let body = response.text().await?;

    ensure_success(status, &body)?;
    parse_translation(&body)
}

fn ensure_success(status: StatusCode, body: &str) -> Result<(), TranslateError> {
    if status.is_success() {
        return Ok(());
    }

    Err(TranslateError::HttpStatus {
        status,
        body: body_excerpt(body),
    })
}

fn body_excerpt(body: &str) -> String {
    let compact = body.split_whitespace().collect::<Vec<_>>().join(" ");
    compact.chars().take(160).collect()
}

fn parse_translation(body: &str) -> Result<String, TranslateError> {
    let json: Value = serde_json::from_str(body)?;
    let segments = json
        .get(0)
        .and_then(Value::as_array)
        .ok_or(TranslateError::MissingTranslation)?;

    let translation = segments
        .iter()
        .filter_map(|segment| segment.get(0).and_then(Value::as_str))
        .collect::<String>();

    if translation.is_empty() {
        Err(TranslateError::MissingTranslation)
    } else {
        Ok(translation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_google_translation_segments() {
        let body =
            r#"[[["Привет, ","Hello, ",null,null,10],["мир!","world!",null,null,10]],null,"en"]"#;

        assert_eq!(parse_translation(body).unwrap(), "Привет, мир!");
    }

    #[test]
    fn rejects_non_json_response() {
        let error = parse_translation("<html>Too many requests</html>").unwrap_err();

        assert!(matches!(error, TranslateError::InvalidJson(_)));
    }

    #[test]
    fn rejects_response_without_translation() {
        let error = parse_translation(r#"[[],null,"en"]"#).unwrap_err();

        assert!(matches!(error, TranslateError::MissingTranslation));
    }

    #[test]
    fn reports_http_error_with_short_compact_body() {
        let error = ensure_success(
            StatusCode::TOO_MANY_REQUESTS,
            "<html>\n  automated queries\n</html>",
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Google Translate returned HTTP 429 Too Many Requests: <html> automated queries </html>"
        );
    }

    #[test]
    #[ignore = "calls the live undocumented Google Translate endpoint"]
    fn translates_live_google_response() {
        assert_eq!(google("translator".into()).unwrap(), "переводчик");
    }
}
