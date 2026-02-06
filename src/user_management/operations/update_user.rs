use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;

use crate::user_management::{PasswordParams, User, UserId, UserManagement};
use crate::{Metadata, ResponseExt, WorkOsError, WorkOsResult};

/// The parameters for [`UpdateUser`].
#[derive(Debug, Serialize)]
pub struct UpdateUserParams<'a> {
    /// The email address of the user.
    pub email: Option<&'a str>,

    /// The password to set for the user.
    #[serde(flatten)]
    pub password: Option<&'a PasswordParams<'a>>,

    /// The first name of the user.
    pub first_name: Option<&'a str>,

    /// The last name of the user.
    pub last_name: Option<&'a str>,

    /// Whether the user's email address was previously verified.
    pub email_verified: Option<bool>,

    /// The external ID of the user.
    pub external_id: Option<&'a str>,

    /// Object containing metadata key/value pairs associated with the user.
    pub metadata: Option<Metadata>,
}

/// An error returned from [`UpdateUser`].
#[derive(Debug, Error)]
pub enum UpdateUserError {}

impl From<UpdateUserError> for WorkOsError<UpdateUserError> {
    fn from(err: UpdateUserError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Update a User](https://workos.com/docs/reference/user-management/user/update)
#[async_trait]
pub trait UpdateUser {
    /// Updates a [`User`].
    ///
    /// [WorkOS Docs: Update a User](https://workos.com/docs/reference/user-management/user/update)
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
    /// # async fn run() -> WorkOsResult<(), UpdateUserError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// let user = workos
    ///     .user_management()
    ///     .update_user(
    ///         &UserId::from("user_01E4ZCR3C56J083X43JQXF3JK5"),
    ///         &UpdateUserParams {
    ///             email: Some("marcelina.updated@example.com"),
    ///             password: None,
    ///             first_name: Some("Marcelina"),
    ///             last_name: Some("Davis-Updated"),
    ///             email_verified: Some(true),
    ///             external_id: None,
    ///             metadata: None,
    ///         },
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn update_user(
        &self,
        user_id: &UserId,
        params: &UpdateUserParams<'_>,
    ) -> WorkOsResult<User, UpdateUserError>;
}

#[async_trait]
impl UpdateUser for UserManagement<'_> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    async fn update_user(
        &self,
        user_id: &UserId,
        params: &UpdateUserParams<'_>,
    ) -> WorkOsResult<User, UpdateUserError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/user_management/users/{user_id}"))?;

        let user = self
            .workos
            .send(
                self.workos
                    .client()
                    .put(url)
                    .bearer_auth(self.workos.key())
                    .json(&params),
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

    use crate::user_management::UserId;
    use crate::{ApiKey, WorkOs};

    use super::*;

    #[tokio::test]
    async fn it_calls_the_update_user_endpoint() {
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
                    "email": "marcelina.updated@example.com",
                    "first_name": "Marcelina",
                    "last_name": "Davis-Updated",
                    "email_verified": true,
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
            .update_user(
                &UserId::from("user_01E4ZCR3C56J083X43JQXF3JK5"),
                &UpdateUserParams {
                    email: Some("marcelina.updated@example.com"),
                    password: None,
                    first_name: Some("Marcelina"),
                    last_name: Some("Davis-Updated"),
                    email_verified: Some(true),
                    external_id: None,
                    metadata: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(user.email, "marcelina.updated@example.com");
        assert_eq!(user.last_name, Some("Davis-Updated".to_string()));
    }
}
