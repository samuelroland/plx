use std::sync::Arc;

use serde::Serialize;
use specta::Type;

use crate::core::diff::diff::Diff;

use super::check::Check;

/// Represents the status of a check
#[derive(Serialize, Debug, Clone, PartialEq, Type)]
#[serde(tag = "type", content = "content")]
pub enum CheckStatus {
    Passed,
    Failed {
        expected: String,
        given: String,
        diff: Diff,
    },
    Checking,
    Running,
    RunFail(String),
    Pending,
}
/// Handles the check and it's current status
#[derive(Serialize, Debug, Clone, PartialEq, Type)]
pub struct CheckState {
    pub(crate) check: Arc<Check>,
    pub(crate) status: CheckStatus,
}
impl CheckState {
    pub(crate) fn new(check: &Check) -> Self {
        Self {
            check: Arc::new(check.clone()),
            status: CheckStatus::Pending,
        }
    }
}
