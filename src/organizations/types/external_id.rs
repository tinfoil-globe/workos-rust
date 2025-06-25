use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

/// The external ID of an organization.
#[derive(
    Clone, Debug, Deref, Display, From, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[from(forward)]
pub struct ExternalId(String);
