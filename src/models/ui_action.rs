use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;

/// An action requested by the UI
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(tag = "type", content = "content")]
pub enum UiAction {
    StartExo {
        exo_folder: PathBuf,
    },
    StopExo,
    /// Stop the App instance and all the workers
    StopApp,
}
