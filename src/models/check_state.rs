use std::sync::Arc;

use serde::Serialize;
use specta_macros::Type;
use typeshare::typeshare;

use crate::core::diff::diff::Diff;

use super::check::Check;

/// Represents the status of a check
#[derive(Serialize, Debug, Clone, PartialEq, Type)]
#[serde(tag = "type", content = "content")]
#[typeshare]
pub enum DetailledCheckStatus {
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
#[typeshare]
pub struct CheckWithStatus {
    // The UI already has the check via the whole course info, do not serialize it again here
    #[serde(skip_serializing)]
    pub(crate) check: Arc<Check>,
    pub(crate) status: DetailledCheckStatus,
}
impl CheckWithStatus {
    pub(crate) fn new(check: &Check) -> Self {
        Self {
            check: Arc::new(check.clone()),
            status: DetailledCheckStatus::Pending,
        }
    }
}
