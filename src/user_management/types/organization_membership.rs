use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Timestamps;
use crate::organizations::OrganizationId;
use crate::user_management::types::user::UserId;

/// The ID of an [`OrganizationMembership`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OrganizationMembershipId(String);

impl Display for OrganizationMembershipId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for OrganizationMembershipId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for OrganizationMembershipId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

/// The state of an [`OrganizationMembership`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrganizationMembershipStatus {
    /// The membership is active.
    Active,

    /// The membership is inactive.
    Inactive,
}

/// [WorkOS Docs: Organization Membership](https://workos.com/docs/reference/user-management/organization-membership)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationMembership {
    /// The ID of the organization membership.
    pub id: OrganizationMembershipId,

    /// The ID of the user.
    pub user_id: UserId,

    /// The ID of the organization.
    pub organization_id: OrganizationId,

    /// The role of the user in the organization.
    pub role: OrganizationRole,

    /// The status of the membership.
    pub status: OrganizationMembershipStatus,

    /// The timestamps for the organization membership.
    #[serde(flatten)]
    pub timestamps: Timestamps,
}

/// The role of a user in an organization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationRole {
    /// The slug of the role.
    pub slug: String,
}
