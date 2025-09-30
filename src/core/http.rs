use std::time::Duration;

use reqwest::{Body, Method, Response, StatusCode, header::HeaderMap};
use url::Url;

pub(crate) const MAX_BODY_LOG_BYTES: usize = 8 * 1024;

#[derive(Clone)]
pub(crate) struct ResponseLogContext {
    pub method: Method,
    pub url: Url,
    pub response_headers: Vec<(String, String)>,
    pub duration: Duration,
}

pub(crate) fn store_response_context(response: &mut Response, context: ResponseLogContext) {
    response.extensions_mut().insert(context);
}

pub(crate) fn response_context(response: &Response) -> Option<ResponseLogContext> {
    response.extensions().get::<ResponseLogContext>().cloned()
}

pub(crate) fn sanitize_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            let value_string = if name.as_str().eq_ignore_ascii_case("authorization") {
                "<redacted>".to_string()
            } else {
                match value.to_str() {
                    Ok(val) => val.to_string(),
                    Err(_) => "<non-utf8-value>".to_string(),
                }
            };

            (name.to_string(), value_string)
        })
        .collect()
}

pub(crate) fn extract_request_body(body: &Body) -> Option<String> {
    match body.as_bytes() {
        Some(bytes) => Some(truncate_for_log(
            &String::from_utf8_lossy(bytes),
            MAX_BODY_LOG_BYTES,
        )),
        None => Some("<non-replayable body>".to_string()),
    }
}

pub(crate) fn truncate_for_log(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }

    let mut truncated = String::with_capacity(limit + 1);
    let mut byte_count = 0;

    for ch in text.chars() {
        let ch_len = ch.len_utf8();
        if byte_count + ch_len > limit {
            break;
        }
        truncated.push(ch);
        byte_count += ch_len;
    }

    truncated.push('…');
    truncated
}

#[cfg(feature = "tracing")]
pub(crate) fn log_request(
    method: &str,
    url: &Url,
    headers: &[(String, String)],
    body: Option<&str>,
) {
    tracing::debug!(
        "sending request",
        http.method = method,
        http.url = %url,
        http.headers = ?headers,
        http.body = body.unwrap_or("<empty>")
    );
}

#[cfg(not(feature = "tracing"))]
pub(crate) fn log_request(
    method: &str,
    url: &Url,
    headers: &[(String, String)],
    body: Option<&str>,
) {
    let _ = (method, url, headers, body);
}

#[cfg(feature = "tracing")]
pub(crate) fn log_response_success(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    duration: Duration,
) {
    tracing::debug!(
        "received response",
        http.method = method,
        http.url = %url,
        http.status = %status,
        http.headers = ?headers,
        elapsed_ms = duration.as_millis()
    );
}

#[cfg(not(feature = "tracing"))]
pub(crate) fn log_response_success(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    duration: Duration,
) {
    let _ = (method, url, status, headers, duration);
}

#[cfg(feature = "tracing")]
pub(crate) fn log_response_status(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    duration: Duration,
) {
    tracing::debug!(
        "received non-success response",
        http.method = method,
        http.url = %url,
        http.status = %status,
        http.headers = ?headers,
        elapsed_ms = duration.as_millis()
    );
}

#[cfg(not(feature = "tracing"))]
pub(crate) fn log_response_status(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    duration: Duration,
) {
    let _ = (method, url, status, headers, duration);
}

#[cfg(feature = "tracing")]
pub(crate) fn log_response_unauthorized(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    duration: Duration,
) {
    tracing::warn!(
        "unauthorized response",
        http.method = method,
        http.url = %url,
        http.status = %status,
        http.headers = ?headers,
        elapsed_ms = duration.as_millis()
    );
}

#[cfg(not(feature = "tracing"))]
pub(crate) fn log_response_unauthorized(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    duration: Duration,
) {
    let _ = (method, url, status, headers, duration);
}

#[cfg(feature = "tracing")]
pub(crate) fn log_response_error_with_body(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    body: &str,
    duration: Duration,
) {
    tracing::error!(
        "error response",
        http.method = method,
        http.url = %url,
        http.status = %status,
        http.headers = ?headers,
        http.body = body,
        elapsed_ms = duration.as_millis()
    );
}

#[cfg(not(feature = "tracing"))]
pub(crate) fn log_response_error_with_body(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    body: &str,
    duration: Duration,
) {
    let _ = (method, url, status, headers, body, duration);
}

#[cfg(feature = "tracing")]
pub(crate) fn log_response_error_body_failed(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    error: &str,
    duration: Duration,
) {
    tracing::error!(
        "error response (body unavailable)",
        http.method = method,
        http.url = %url,
        http.status = %status,
        http.headers = ?headers,
        error,
        elapsed_ms = duration.as_millis()
    );
}

#[cfg(not(feature = "tracing"))]
pub(crate) fn log_response_error_body_failed(
    method: &str,
    url: &Url,
    status: StatusCode,
    headers: &[(String, String)],
    error: &str,
    duration: Duration,
) {
    let _ = (method, url, status, headers, error, duration);
}
