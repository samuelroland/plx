use std::{path::PathBuf, sync::Arc};

use serde::{Serialize, Serializer};
use specta_macros::Type;

use crate::models::{check_state::CheckWithStatus, exo::Exo};

use super::exo_check_result::ExoCheckResultWithOutput;

#[derive(Serialize, Clone, Type)]
#[typeshare::typeshare]
pub struct FileContent {
    /// The relative path inside the exo folder, like "main.cpp", "src/main.rs", "lib/image.h"
    pub path: String,
    pub content: String,
}

/// ExoStatusReport
///
/// This struct is used to store the result of a run + check
/// It keeps the information of an exo run, including the check results,
/// the compilation output and the path to the elf file
/// See `ExoCheckResult` for more information about the check results
#[derive(Serialize, Clone, Type)]
#[typeshare::typeshare]
pub struct ExoStatusReport {
    pub(super) check_results: Vec<ExoCheckResultWithOutput>,
    #[serde(serialize_with = "terminal_lines_to_html")]
    pub(super) compilation_output: String,
    pub(super) compilation_success: bool,
    pub(super) compilation_running: bool,
    #[serde(skip_serializing)]
    pub(super) elf_path: PathBuf,
    #[serde(skip_serializing)]
    pub(super) exo: Arc<Exo>,
    /// A vector with content of edited files, among the list of code files for this exo
    pub(super) edited_files_content: Vec<FileContent>,
}

/// Serialize terminal lines as HTML, by converting ANSI codes to HTML equivalent
/// The HTML is styled with inline CSS and doesn't need any external CSS definitions
fn terminal_lines_to_html<S>(lines: &str, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // TODO: could this cause XSS ?? should this be sanitized with ammonia ?
    let converter = ansi_to_html::Converter::new();
    let result = converter.convert(lines).unwrap_or_else(|e| e.to_string());
    s.serialize_str(&result)
}

impl ExoStatusReport {
    /// Create an ExoStatusReport from an Exo and the target path
    pub(super) fn new(exo: &Exo, elf_path: PathBuf) -> Self {
        let checkers: Vec<ExoCheckResultWithOutput> = exo
            .checks
            .iter()
            .map(ExoCheckResultWithOutput::new)
            .collect();

        Self {
            check_results: checkers,
            compilation_output: String::new(),
            compilation_success: false,
            compilation_running: false,
            elf_path,
            exo: Arc::new(exo.clone()),
            edited_files_content: vec![],
        }
    }

    /// Helper function to get a `Vec<CheckState>` from check results
    /// Useful to send the check states to the Ui
    /// Check `UiState::CheckResults` for more information
    pub(super) fn to_vec_check_state(&self) -> Vec<CheckWithStatus> {
        self.check_results
            .iter()
            .map(|result| result.state.clone())
            .collect()
    }
}
