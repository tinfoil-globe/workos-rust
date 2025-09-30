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

pub(crate) fn collect_error_chain(err: &reqwest::Error) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current: &(dyn std::error::Error + 'static) = err;
    while let Some(source) = current.source() {
        let source_string = source.to_string();
        if chain
            .last()
            .map(|prev| prev == &source_string)
            .unwrap_or(false)
        {
            current = source;
            continue;
        }

        chain.push(source_string);
        current = source;
    }
    chain
}

pub(crate) fn derive_error_hint(err: &reqwest::Error, chain: &[String]) -> Option<String> {
    let mut messages = Vec::with_capacity(chain.len() + 1);
    messages.push(err.to_string());
    messages.extend(chain.iter().cloned());

    let combined = messages.join(" | ").to_lowercase();

    if err.is_timeout() || combined.contains("timed out") {
        return Some("Connection timed out while contacting WorkOS".to_string());
    }

    if combined.contains("dns error")
        || combined.contains("failed to lookup address information")
        || combined.contains("failed to resolve")
    {
        return Some("DNS resolution failed for the WorkOS endpoint".to_string());
    }

    if combined.contains("connection refused") {
        return Some("Remote host refused the TCP connection".to_string());
    }

    if combined.contains("certificate verify failed")
        || combined.contains("unable to get local issuer certificate")
    {
        return Some(
            "TLS certificate verification failed; ensure the trust store is available".to_string(),
        );
    }

    if combined.contains("ossl_store_get0_loader_int") || combined.contains("unregistered scheme") {
        return Some(
            "OpenSSL certificate store loader is unavailable; check OpenSSL providers/config"
                .to_string(),
        );
    }

    None
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
        method = tracing::field::display(method),
        url = tracing::field::display(url),
        request_headers = tracing::field::debug(headers),
        request_body = body.unwrap_or("<empty>"),
        "sending request"
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn log_request_failure(
    method: &str,
    url: &Url,
    headers: &[(String, String)],
    body: Option<&str>,
    duration: Duration,
    err: &reqwest::Error,
    error_causes: &[String],
    error_hint: Option<&str>,
) {
    match error_hint {
        Some(hint) => tracing::error!(
            method = tracing::field::display(method),
            url = tracing::field::display(url),
            request_headers = tracing::field::debug(headers),
            request_body = body.unwrap_or("<empty>"),
            elapsed_ms = duration.as_millis(),
            error = tracing::field::display(err),
            error_is_timeout = err.is_timeout(),
            error_is_request = err.is_request(),
            error_is_connect = err.is_connect(),
            error_is_body = err.is_body(),
            error_is_decode = err.is_decode(),
            error_is_builder = err.is_builder(),
            error_chain = tracing::field::debug(error_causes),
            error_hint = tracing::field::display(hint),
            "request failed"
        ),
        None => tracing::error!(
            method = tracing::field::display(method),
            url = tracing::field::display(url),
            request_headers = tracing::field::debug(headers),
            request_body = body.unwrap_or("<empty>"),
            elapsed_ms = duration.as_millis(),
            error = tracing::field::display(err),
            error_is_timeout = err.is_timeout(),
            error_is_request = err.is_request(),
            error_is_connect = err.is_connect(),
            error_is_body = err.is_body(),
            error_is_decode = err.is_decode(),
            error_is_builder = err.is_builder(),
            error_chain = tracing::field::debug(error_causes),
            "request failed"
        ),
    }
}

#[cfg(not(feature = "tracing"))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn log_request_failure(
    method: &str,
    url: &Url,
    headers: &[(String, String)],
    body: Option<&str>,
    duration: Duration,
    err: &reqwest::Error,
    error_causes: &[String],
    error_hint: Option<&str>,
) {
    let _ = (
        method,
        url,
        headers,
        body,
        duration,
        err,
        error_causes,
        error_hint,
    );
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
        method = tracing::field::display(method),
        url = tracing::field::display(url),
        status = tracing::field::display(status),
        response_headers = tracing::field::debug(headers),
        elapsed_ms = duration.as_millis(),
        "received response"
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
        method = tracing::field::display(method),
        url = tracing::field::display(url),
        status = tracing::field::display(status),
        response_headers = tracing::field::debug(headers),
        elapsed_ms = duration.as_millis(),
        "received non-success response"
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
        method = tracing::field::display(method),
        url = tracing::field::display(url),
        status = tracing::field::display(status),
        response_headers = tracing::field::debug(headers),
        elapsed_ms = duration.as_millis(),
        "unauthorized response"
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
        method = tracing::field::display(method),
        url = tracing::field::display(url),
        status = tracing::field::display(status),
        response_headers = tracing::field::debug(headers),
        response_body = body,
        elapsed_ms = duration.as_millis(),
        "error response"
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
        method = tracing::field::display(method),
        url = tracing::field::display(url),
        status = tracing::field::display(status),
        response_headers = tracing::field::debug(headers),
        error = tracing::field::display(error),
        elapsed_ms = duration.as_millis(),
        "error response (body unavailable)"
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
