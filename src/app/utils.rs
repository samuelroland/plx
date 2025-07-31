use crate::models::check_state::{CheckWithStatus, DetailledCheckStatus};

use super::app::App;

impl App {
    /// Checks if all checks in `checks` have passed
    pub(super) fn all_checks_passed(checks: &Vec<CheckWithStatus>) -> bool {
        checks
            .iter()
            .all(|result| result.status == DetailledCheckStatus::Passed)
    }
}
