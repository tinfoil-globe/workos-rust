use async_trait::async_trait;
use serde_json::json;
use thiserror::Error;

use crate::organizations::{ExternalId, Organization, OrganizationId, Organizations};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// An error returned from [`UpdateExternalId`].
#[derive(Debug, Error)]
pub enum UpdateExternalIdError {}

impl From<UpdateExternalIdError> for WorkOsError<UpdateExternalIdError> {
    fn from(err: UpdateExternalIdError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Update an Organization's External ID](https://workos.com/docs/reference/organization/update)
#[async_trait]
pub trait UpdateExternalId {
    /// Updates an organization's external ID.
    ///
    /// [WorkOS Docs: Update an Organization's External ID](https://workos.com/docs/reference/organization/update)
    ///
    /// # Examples
    ///
    /// ```
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::organizations::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), UpdateExternalIdError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let organization = workos
    ///     .organizations()
    ///     .update_external_id(
    ///         &OrganizationId::from("org_01EHZNVPK3SFK441A1RGBFSHRT"),
    ///         &ExternalId::from("external_12345"),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn update_external_id(
        &self,
        organization_id: &OrganizationId,
        external_id: &ExternalId,
    ) -> WorkOsResult<Organization, UpdateExternalIdError>;
}

#[async_trait]
impl UpdateExternalId for Organizations<'_> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    async fn update_external_id(
        &self,
        organization_id: &OrganizationId,
        external_id: &ExternalId,
    ) -> WorkOsResult<Organization, UpdateExternalIdError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/organizations/{organization_id}"))?;

        let body = json!({
            "external_id": external_id
        });

        let organization = self
            .workos
            .send(
                self.workos
                    .client()
                    .put(url)
                    .bearer_auth(self.workos.key())
                    .json(&body),
            )
            .await?
            .handle_unauthorized_or_generic_error()
            .await?
            .json::<Organization>()
            .await?;

        Ok(organization)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use crate::organizations::{ExternalId, OrganizationId};
    use crate::{ApiKey, WorkOs};

    use super::*;

    #[tokio::test]
    async fn it_calls_the_update_external_id_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock("PUT", "/organizations/org_01EHZNVPK3SFK441A1RGBFSHRT")
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "id": "org_01EHZNVPK3SFK441A1RGBFSHRT",
                    "object": "organization",
                    "name": "Foo Corp",
                    "external_id": "external_12345",
                    "allow_profiles_outside_organization": false,
                    "created_at": "2021-06-25T19:07:33.155Z",
                    "updated_at": "2021-06-25T20:07:33.155Z",
                    "domains": [
                        {
                            "domain": "foo-corp.com",
                            "id": "org_domain_01EHZNVPK2QXHMVWCEDQEKY69A",
                            "object": "organization_domain"
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let organization = workos
            .organizations()
            .update_external_id(
                &OrganizationId::from("org_01EHZNVPK3SFK441A1RGBFSHRT"),
                &ExternalId::from("external_12345"),
            )
            .await
            .unwrap();

        assert_eq!(
            organization.id,
            OrganizationId::from("org_01EHZNVPK3SFK441A1RGBFSHRT")
        );
        assert_eq!(
            organization.external_id,
            Some(ExternalId::from("external_12345"))
        );
    }
}
