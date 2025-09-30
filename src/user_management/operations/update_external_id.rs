use async_trait::async_trait;
use serde_json::json;
use thiserror::Error;

use crate::user_management::{ExternalId, User, UserId, UserManagement};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// An error returned from [`UpdateExternalId`].
#[derive(Debug, Error)]
pub enum UpdateExternalIdError {}

impl From<UpdateExternalIdError> for WorkOsError<UpdateExternalIdError> {
    fn from(err: UpdateExternalIdError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Update a User's External ID](https://workos.com/docs/reference/user-management/user/update)
#[async_trait]
pub trait UpdateExternalId {
    /// Updates a user's external ID.
    ///
    /// [WorkOS Docs: Update a User's External ID](https://workos.com/docs/reference/user-management/user/update)
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// # use workos_sdk::WorkOsResult;
    /// # use workos_sdk::user_management::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), UpdateExternalIdError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let user = workos
    ///     .user_management()
    ///     .update_external_id(
    ///         &UserId::from("user_01E4ZCR3C56J083X43JQXF3JK5"),
    ///         &ExternalId::from("external_12345"),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn update_external_id(
        &self,
        user_id: &UserId,
        external_id: &ExternalId,
    ) -> WorkOsResult<User, UpdateExternalIdError>;
}

#[async_trait]
impl UpdateExternalId for UserManagement<'_> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    async fn update_external_id(
        &self,
        user_id: &UserId,
        external_id: &ExternalId,
    ) -> WorkOsResult<User, UpdateExternalIdError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/user_management/users/{user_id}"))?;

        let body = json!({
            "external_id": external_id
        });

        let user = self
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
            .json::<User>()
            .await?;

        Ok(user)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use tokio;

    use crate::user_management::{ExternalId, UserId};
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
            .mock(
                "PUT",
                "/user_management/users/user_01E4ZCR3C56J083X43JQXF3JK5",
            )
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(200)
            .with_body(
                json!({
                    "object": "user",
                    "id": "user_01E4ZCR3C56J083X43JQXF3JK5",
                    "email": "marcelina.davis@example.com",
                    "first_name": "Marcelina",
                    "last_name": "Davis",
                    "email_verified": true,
                    "external_id": "external_12345",
                    "profile_picture_url": "https://workoscdn.com/images/v1/123abc",
                    "last_sign_in_at": "2021-06-25T19:07:33.155Z",
                    "created_at": "2021-06-25T19:07:33.155Z",
                    "updated_at": "2021-06-25T20:07:33.155Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let user = workos
            .user_management()
            .update_external_id(
                &UserId::from("user_01E4ZCR3C56J083X43JQXF3JK5"),
                &ExternalId::from("external_12345"),
            )
            .await
            .unwrap();

        assert_eq!(user.external_id, Some("external_12345".to_string()));
    }
}
