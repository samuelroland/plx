use serde::Serialize;
use specta_macros::Type;
use typeshare::typeshare;

use crate::models::{check::Check, check_state::CheckWithStatus};

/// ExoCheckResult
///
/// This struct is used to store the result of a run + check
/// Each exo run will have as many ExoCheckResults as the number of checks the exo has
/// This helps us keep the output of the run and the check state together
#[derive(Serialize, Clone, Type)]
#[typeshare]
pub(super) struct ExoCheckResultWithOutput {
    pub(super) state: CheckWithStatus,
    pub(super) output: Vec<String>,
}

impl ExoCheckResultWithOutput {
    /// Create an ExoCheckResult from a Check
    pub(super) fn new(check: &Check) -> Self {
        Self {
            state: CheckWithStatus::new(check),
            output: Vec::new(),
        }
    }
}
