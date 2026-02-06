use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Response, StatusCode};
use url::Url;

use crate::core::{
    MAX_BODY_LOG_BYTES, ResponseLogContext, log_response_error_body_failed,
    log_response_error_with_body, log_response_unauthorized, response_context, sanitize_headers,
    truncate_for_log,
};
use crate::{RequestError, WorkOsError, WorkOsResult};

#[async_trait]
pub trait ResponseExt
where
    Self: Sized + Send,
{
    /// Handles an unauthorized error from the WorkOS API by converting it into a
    /// [`WorkOsError::Unauthorized`] response.
    async fn handle_unauthorized_error<E: Send>(self) -> WorkOsResult<Self, E>;

    /// Handles a generic error from the WorkOS API by converting it into a
    /// [`WorkOsError::RequestError`] response with additional context.
    async fn handle_generic_error<E: Send>(self) -> WorkOsResult<Self, E>;

    /// Handles an unauthorized or generic error from the WorkOS API.
    async fn handle_unauthorized_or_generic_error<E: Send>(self) -> WorkOsResult<Self, E>;
}

#[async_trait]
impl ResponseExt for Response {
    async fn handle_unauthorized_error<E: Send>(self) -> WorkOsResult<Self, E> {
        if self.status() == StatusCode::UNAUTHORIZED {
            if let Some(context) = response_context(&self) {
                log_response_unauthorized(
                    context.method.as_str(),
                    &context.url,
                    self.status(),
                    &context.response_headers,
                    context.duration,
                );
            } else {
                log_response_unauthorized(
                    "UNKNOWN",
                    self.url(),
                    self.status(),
                    &sanitize_headers(self.headers()),
                    Duration::default(),
                );
            }

            Err(WorkOsError::Unauthorized)
        } else {
            Ok(self)
        }
    }

    async fn handle_generic_error<E: Send>(self) -> WorkOsResult<Self, E> {
        if self.status().is_success() {
            Ok(self)
        } else {
            Err(response_to_request_error(self).await)
        }
    }

    async fn handle_unauthorized_or_generic_error<E: Send>(self) -> WorkOsResult<Self, E> {
        self.handle_unauthorized_error()
            .await?
            .handle_generic_error()
            .await
    }
}

pub(crate) async fn response_to_request_error<E>(response: Response) -> WorkOsError<E> {
    let status = response.status();
    let context = response_context(&response);
    let context_clone = context.clone();
    let fallback_url = response.url().clone();
    let fallback_headers = sanitize_headers(response.headers());

    match response.text().await {
        Ok(body) => {
            let truncated = truncate_for_log(&body, MAX_BODY_LOG_BYTES);
            build_request_error_from_body(
                context,
                &fallback_url,
                &fallback_headers,
                status,
                &truncated,
            )
        }
        Err(err) => {
            let display_err = err.to_string();
            let (method, url_ref, headers_ref, duration) = context_clone
                .as_ref()
                .map(|ctx| {
                    (
                        ctx.method.as_str(),
                        &ctx.url,
                        ctx.response_headers.as_slice(),
                        ctx.duration,
                    )
                })
                .unwrap_or((
                    "UNKNOWN",
                    &fallback_url,
                    fallback_headers.as_slice(),
                    Duration::default(),
                ));
            log_response_error_body_failed(
                method,
                url_ref,
                status,
                headers_ref,
                &display_err,
                duration,
            );
            let message = format!(
                "{} {} returned {} but the response body could not be read: {}",
                method, url_ref, status, display_err
            );

            WorkOsError::RequestError(RequestError::with_source(message, err))
        }
    }
}

fn format_error_message(method: &str, url: &Url, status: StatusCode, body: &str) -> String {
    if body.is_empty() {
        format!("{} {} returned {} with empty body", method, url, status)
    } else {
        format!("{} {} returned {} with body: {}", method, url, status, body)
    }
}

pub(crate) fn build_request_error_from_body<E>(
    context: Option<ResponseLogContext>,
    fallback_url: &Url,
    fallback_headers: &[(String, String)],
    status: StatusCode,
    body: &str,
) -> WorkOsError<E> {
    match context {
        Some(ctx) => {
            log_response_error_with_body(
                ctx.method.as_str(),
                &ctx.url,
                status,
                &ctx.response_headers,
                body,
                ctx.duration,
            );
            let message = format_error_message(ctx.method.as_str(), &ctx.url, status, body);
            WorkOsError::RequestError(RequestError::new(message))
        }
        None => {
            log_response_error_with_body(
                "UNKNOWN",
                fallback_url,
                status,
                fallback_headers,
                body,
                Duration::default(),
            );
            let message = format_error_message("UNKNOWN", fallback_url, status, body);
            WorkOsError::RequestError(RequestError::new(message))
        }
    }
}
