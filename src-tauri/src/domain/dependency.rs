use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencySource {
    /// A `skills` executable was found directly on the (augmented) PATH.
    InstalledExecutable,
    /// Falling back to `npx --yes skills`.
    Npx,
    /// Nothing usable was found.
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyStatus {
    pub available: bool,
    pub source: DependencySource,
    pub executable_path: Option<String>,
    pub version: Option<String>,
    pub detail: Option<String>,
}
