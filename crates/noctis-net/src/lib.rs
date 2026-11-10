//! HTTP(S) transport boundary for NOCTIS.
//!
//! This crate implements URL validation and bounded document fetching. It does
//! not parse HTML, execute scripts, apply CSS, or render pages.

use std::io::Read;
use std::time::Duration;
use url::Url;

pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_REDIRECTS: u32 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetError {
    InvalidUrl(String),
    UnsupportedScheme(String),
    MissingHost,
    CredentialsNotAllowed,
    ResponseTooLarge { limit: usize },
    HttpStatus(u16),
    Transport(String),
    Io(String),
}

impl std::fmt::Display for NetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(message) => write!(formatter, "invalid URL: {message}"),
            Self::UnsupportedScheme(scheme) => {
                write!(formatter, "unsupported URL scheme: {scheme}")
            }
            Self::MissingHost => write!(formatter, "URL has no host"),
            Self::CredentialsNotAllowed => write!(
                formatter,
                "URLs containing embedded credentials are not allowed"
            ),
            Self::ResponseTooLarge { limit } => {
                write!(formatter, "response exceeds the {limit}-byte limit")
            }
            Self::HttpStatus(status) => write!(formatter, "server returned HTTP {status}"),
            Self::Transport(message) => write!(formatter, "network error: {message}"),
            Self::Io(message) => write!(formatter, "response read error: {message}"),
        }
    }
}

impl std::error::Error for NetError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResponse {
    pub requested_url: String,
    pub final_url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

/// Parse and validate a navigable web URL. Only HTTP and HTTPS are accepted.
pub fn parse_web_url(input: &str) -> Result<Url, NetError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(NetError::InvalidUrl("URL is empty".to_owned()));
    }
    if trimmed.chars().any(char::is_control) {
        return Err(NetError::InvalidUrl(
            "URL contains control characters".to_owned(),
        ));
    }

    let url = Url::parse(trimmed).map_err(|error| NetError::InvalidUrl(error.to_string()))?;
    match url.scheme() {
        "http" | "https" => {}
        other => return Err(NetError::UnsupportedScheme(other.to_owned())),
    }
    if url.host_str().is_none() {
        return Err(NetError::MissingHost);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(NetError::CredentialsNotAllowed);
    }
    Ok(url)
}

fn read_bounded<R: Read>(reader: R, limit: usize) -> Result<Vec<u8>, NetError> {
    let mut bytes = Vec::new();
    let read_limit = u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1);
    reader
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| NetError::Io(error.to_string()))?;
    if bytes.len() > limit {
        return Err(NetError::ResponseTooLarge { limit });
    }
    Ok(bytes)
}

/// Fetch an HTTP(S) resource with timeouts, bounded redirects and a 5 MiB body cap.
pub fn fetch_url(input: &str) -> Result<FetchResponse, NetError> {
    fetch_url_with_limit(input, DEFAULT_MAX_RESPONSE_BYTES)
}

/// Fetch a resource with a caller-selected response-size limit.
pub fn fetch_url_with_limit(
    input: &str,
    max_response_bytes: usize,
) -> Result<FetchResponse, NetError> {
    let requested = parse_web_url(input)?;
    let requested_url = requested.to_string();

    let agent = ureq::AgentBuilder::new()
        .timeout(REQUEST_TIMEOUT)
        .timeout_connect(REQUEST_TIMEOUT)
        .redirects(MAX_REDIRECTS)
        .build();

    let response = match agent
        .get(requested.as_str())
        .set("User-Agent", "NOCTIS/0.1.0 (independent browser engine)")
        .set(
            "Accept",
            "text/html,application/xhtml+xml,text/plain;q=0.8,*/*;q=0.1",
        )
        .call()
    {
        Ok(response) => response,
        Err(ureq::Error::Status(status, _response)) => return Err(NetError::HttpStatus(status)),
        Err(error) => return Err(NetError::Transport(error.to_string())),
    };

    if let Some(length) = response
        .header("Content-Length")
        .and_then(|value| value.parse::<usize>().ok())
    {
        if length > max_response_bytes {
            return Err(NetError::ResponseTooLarge {
                limit: max_response_bytes,
            });
        }
    }

    let final_url = response.get_url().to_owned();
    let status = response.status();
    let content_type = response.header("Content-Type").map(str::to_owned);
    let body = read_bounded(response.into_reader(), max_response_bytes)?;

    Ok(FetchResponse {
        requested_url,
        final_url,
        status,
        content_type,
        body,
    })
}

/// Resolve a link against a document URL while enforcing the HTTP(S)-only policy.
pub fn resolve_reference(base: &str, reference: &str) -> Result<String, NetError> {
    let base = parse_web_url(base)?;
    let resolved = base
        .join(reference.trim())
        .map_err(|error| NetError::InvalidUrl(error.to_string()))?;
    parse_web_url(resolved.as_str())?;
    Ok(resolved.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn accepts_http_and_https_urls() {
        assert_eq!(
            parse_web_url("https://example.com/path?q=1")
                .unwrap()
                .host_str(),
            Some("example.com")
        );
        assert_eq!(
            parse_web_url("http://127.0.0.1:8080/").unwrap().scheme(),
            "http"
        );
    }

    #[test]
    fn rejects_non_web_schemes_and_embedded_credentials() {
        assert!(matches!(
            parse_web_url("file:///etc/passwd"),
            Err(NetError::UnsupportedScheme(_))
        ));
        assert_eq!(
            parse_web_url("https://user:secret@example.com"),
            Err(NetError::CredentialsNotAllowed)
        );
    }

    #[test]
    fn rejects_empty_control_and_hostless_urls() {
        assert!(matches!(parse_web_url("  "), Err(NetError::InvalidUrl(_))));
        assert!(matches!(
            parse_web_url("https://example.com/\nheader"),
            Err(NetError::InvalidUrl(_))
        ));
        assert!(matches!(
            parse_web_url("https://"),
            Err(NetError::InvalidUrl(_)) | Err(NetError::MissingHost)
        ));
    }

    #[test]
    fn fetches_a_local_http_document_end_to_end() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let body = "<html><title>Local</title><p>NOCTIS transport works</p></html>";
        let server_body = body.to_owned();

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request);
            write!(
                stream,
                "HTTP/1.1 200 OK\\r\\nContent-Type: text/html; charset=utf-8\\r\\nContent-Length: {}\\r\\nConnection: close\\r\\n\\r\\n{}",
                server_body.len(),
                server_body
            )
            .unwrap();
        });

        let response = fetch_url_with_limit(&format!("http://{address}/"), 4096).unwrap();
        server.join().unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(response.content_type.as_deref(), Some("text/html; charset=utf-8"));
        assert_eq!(String::from_utf8(response.body).unwrap(), body);
        assert_eq!(response.final_url, format!("http://{address}/"));
    }

    #[test]
    fn reads_body_with_a_hard_size_limit() {
        assert_eq!(read_bounded(Cursor::new(b"hello"), 5).unwrap(), b"hello");
        assert_eq!(
            read_bounded(Cursor::new(b"hello!"), 5),
            Err(NetError::ResponseTooLarge { limit: 5 })
        );
    }
}
