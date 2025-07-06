use crate::models::check_state::{CheckState, CheckStatus};

use super::app::App;

impl App {
    /// Checks if all checks in `checks` have passed
    pub(super) fn all_checks_passed(checks: &Vec<CheckState>) -> bool {
        checks
            .iter()
            .all(|result| result.status == CheckStatus::Passed)
    }
}
