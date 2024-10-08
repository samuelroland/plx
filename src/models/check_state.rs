use std::sync::Arc;

use crate::core::diff::diff::Diff;

use super::check::Check;

/// Represents the status of a check
#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Passed,
    Failed(String, String, Diff),
    Checking,
    Running,
    RunFail(String),
    Pending,
}
/// Handles the check and it's current status
#[derive(Debug, Clone, PartialEq)]
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
