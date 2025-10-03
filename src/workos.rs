use std::time::Instant;

use reqwest::{header::RETRY_AFTER, RequestBuilder, Response, StatusCode};
use url::{ParseError, Url};

use crate::admin_portal::AdminPortal;
use crate::core::{
    ResponseLogContext, extract_request_body, log_request, log_response_status,
    log_response_success, sanitize_headers, store_response_context,
};
use crate::directory_sync::DirectorySync;
use crate::mfa::Mfa;
use crate::organizations::Organizations;
use crate::passwordless::Passwordless;
use crate::roles::Roles;
use crate::sso::Sso;
use crate::user_management::UserManagement;
use crate::{ApiKey, WorkOsError, WorkOsResult};

/// The WorkOS client.
#[derive(Clone)]
pub struct WorkOs {
    base_url: Url,
    key: ApiKey,
    client: reqwest::Client,
}

impl WorkOs {
    /// Returns a new instance of the WorkOS client using the provided API key.
    pub fn new(key: &ApiKey) -> Self {
        WorkOsBuilder::new(key).build()
    }

    /// Returns a [`WorkOsBuilder`] that may be used to construct a WorkOS client.
    pub fn builder(key: &ApiKey) -> WorkOsBuilder<'_> {
        WorkOsBuilder::new(key)
    }

    pub(crate) fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub(crate) fn key(&self) -> &ApiKey {
        &self.key
    }

    pub(crate) fn client(&self) -> &reqwest::Client {
        &self.client
    }

    pub(crate) async fn send<E>(&self, builder: RequestBuilder) -> WorkOsResult<Response, E> {
        let timer = Instant::now();
        let request = builder.build()?;
        let method = request.method().clone();
        let url = request.url().clone();
        let request_headers = sanitize_headers(request.headers());
        let request_body = request.body().and_then(extract_request_body);
        log_request(
            method.as_str(),
            &url,
            &request_headers,
            request_body.as_deref(),
        );

        let mut response = match self.client.execute(request).await {
            Ok(response) => response,
            Err(err) => {
                let duration = timer.elapsed();
                let error_chain = crate::core::collect_error_chain(&err);
                let error_hint = crate::core::derive_error_hint(&err, &error_chain);
                crate::core::log_request_failure(
                    method.as_str(),
                    &url,
                    &request_headers,
                    request_body.as_deref(),
                    duration,
                    &err,
                    &error_chain,
                    error_hint.as_deref(),
                );
                return Err(WorkOsError::from(err));
            }
        };
        let duration = timer.elapsed();
        let status = response.status();
        let response_headers = sanitize_headers(response.headers());

        store_response_context(
            &mut response,
            ResponseLogContext {
                method: method.clone(),
                url: url.clone(),
                response_headers: response_headers.clone(),
                duration,
            },
        );

        if status.is_success() {
            log_response_success(method.as_str(), &url, status, &response_headers, duration);
        } else {
            log_response_status(method.as_str(), &url, status, &response_headers, duration);
        }

        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<f32>().ok());

            return Err(WorkOsError::RateLimited { retry_after });
        }

        Ok(response)
    }

    /// Returns an [`AdminPortal`] instance.
    pub fn admin_portal(&self) -> AdminPortal<'_> {
        AdminPortal::new(self)
    }

    /// Returns a [`DirectorySync`] instance.
    pub fn directory_sync(&self) -> DirectorySync<'_> {
        DirectorySync::new(self)
    }

    /// Returns an [`Mfa`] instance.
    pub fn mfa(&self) -> Mfa<'_> {
        Mfa::new(self)
    }

    /// Returns an [`Organizations`] instance.
    pub fn organizations(&self) -> Organizations<'_> {
        Organizations::new(self)
    }

    /// Returns a [`Passwordless`] instance.
    pub fn passwordless(&self) -> Passwordless<'_> {
        Passwordless::new(self)
    }

    /// Returns a [`Roles`] instance.
    pub fn roles(&self) -> Roles<'_> {
        Roles::new(self)
    }

    /// Returns an [`Sso`] instance.
    pub fn sso(&self) -> Sso<'_> {
        Sso::new(self)
    }

    /// Returns a [`UserManagement`] instance.
    pub fn user_management(&self) -> UserManagement<'_> {
        UserManagement::new(self)
    }
}

/// A builder for a WorkOS client.
pub struct WorkOsBuilder<'a> {
    base_url: Url,
    key: &'a ApiKey,
}

impl<'a> WorkOsBuilder<'a> {
    /// Returns a new [`WorkOsBuilder`] using the provided API key.
    pub fn new(key: &'a ApiKey) -> Self {
        Self {
            base_url: Url::parse("https://api.workos.com").unwrap(),
            key,
        }
    }

    /// Sets the base URL of the WorkOS API that the client should point to.
    pub fn base_url(mut self, base_url: &'a str) -> Result<Self, ParseError> {
        self.base_url = Url::parse(base_url)?;
        Ok(self)
    }

    /// Sets the API key that the client will use.
    pub fn key(mut self, key: &'a ApiKey) -> Self {
        self.key = key;
        self
    }

    /// Consumes the builder and returns the constructed client.
    pub fn build(self) -> WorkOs {
        let client = reqwest::Client::builder()
            .user_agent(concat!("workos-rust/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap();

        WorkOs {
            base_url: self.base_url,
            key: self.key.to_owned(),
            client,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use matches::assert_matches;

    #[test]
    fn it_supports_setting_the_base_url_through_the_builder() {
        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url("https://auth.your-app.com")
            .unwrap()
            .build();

        assert_eq!(
            workos.base_url(),
            &Url::parse("https://auth.your-app.com").unwrap()
        )
    }

    #[test]
    fn it_supports_setting_the_api_key_through_the_builder() {
        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .key(&ApiKey::from("sk_another_api_key"))
            .build();

        assert_eq!(workos.key(), &ApiKey::from("sk_another_api_key"))
    }

    #[tokio::test]
    async fn it_sets_the_user_agent_header_on_the_client() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/health")
            .match_header(
                "User-Agent",
                concat!("workos-rust/", env!("CARGO_PKG_VERSION")),
            )
            .with_status(200)
            .with_body("User-Agent correctly set")
            .create_async()
            .await;

        let url = workos.base_url().join("/health").unwrap();
        let response = workos.client().get(url).send().await.unwrap();
        let response_body = response.text().await.unwrap();

        assert_eq!(response_body, "User-Agent correctly set")
    }

    #[tokio::test]
    async fn it_returns_a_rate_limited_error_with_retry_after() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("GET", "/rate-limited")
            .with_status(429)
            .with_header("Retry-After", "1.5")
            .create_async()
            .await;

        let url = workos.base_url().join("/rate-limited").unwrap();
        let result = workos
            .send::<()>(workos.client().get(url))
            .await;

        assert_matches!(
            result,
            Err(WorkOsError::RateLimited {
                retry_after: Some(value),
            }) if (value - 1.5).abs() < f32::EPSILON
        );
    }
}
