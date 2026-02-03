use async_trait::async_trait;
use thiserror::Error;

use crate::user_management::{UserId, UserManagement};
use crate::{ResponseExt, WorkOsError, WorkOsResult};

/// An error returned from [`DeleteUser`].
#[derive(Debug, Error)]
pub enum DeleteUserError {}

impl From<DeleteUserError> for WorkOsError<DeleteUserError> {
    fn from(err: DeleteUserError) -> Self {
        Self::Operation(err)
    }
}

/// [WorkOS Docs: Delete a User](https://workos.com/docs/reference/user-management/user/delete)
#[async_trait]
pub trait DeleteUser {
    /// Permanently deletes a [`User`](crate::user_management::User).
    ///
    /// [WorkOS Docs: Delete a User](https://workos.com/docs/reference/user-management/user/delete)
    ///
    /// # Examples
    ///
    /// ```
    /// use workos_sdk::WorkOsResult;
    /// use workos_sdk::user_management::*;
    /// use workos_sdk::{ApiKey, WorkOs};
    ///
    /// # async fn run() -> WorkOsResult<(), DeleteUserError> {
    /// let workos = WorkOs::new(&ApiKey::from("sk_example_123456789"));
    ///
    /// workos
    ///     .user_management()
    ///     .delete_user(&UserId::from("user_01E4ZCR3C56J083X43JQXF3JK5"))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_user(&self, user_id: &UserId) -> WorkOsResult<(), DeleteUserError>;
}

#[async_trait]
impl DeleteUser for UserManagement<'_> {
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
    async fn delete_user(&self, user_id: &UserId) -> WorkOsResult<(), DeleteUserError> {
        let url = self
            .workos
            .base_url()
            .join(&format!("/user_management/users/{user_id}"))?;
        self.workos
            .send(
                self.workos
                    .client()
                    .delete(url)
                    .bearer_auth(self.workos.key()),
            )
            .await?
            .handle_unauthorized_or_generic_error()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tokio;

    use super::*;
    use crate::{ApiKey, WorkOs};
    use matches::assert_matches;

    #[tokio::test]
    async fn it_calls_the_delete_user_endpoint() {
        let mut server = mockito::Server::new_async().await;

        let workos = WorkOs::builder(&ApiKey::from("sk_example_123456789"))
            .base_url(&server.url())
            .unwrap()
            .build();

        server
            .mock(
                "DELETE",
                "/user_management/users/user_01E4ZCR3C56J083X43JQXF3JK5",
            )
            .match_header("Authorization", "Bearer sk_example_123456789")
            .with_status(202)
            .create_async()
            .await;

        let result = workos
            .user_management()
            .delete_user(&UserId::from("user_01E4ZCR3C56J083X43JQXF3JK5"))
            .await;

        assert_matches!(result, Ok(()));
    }
}
