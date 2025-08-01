use std::path::PathBuf;

use crate::core::diff::diff::Diff;

use super::ui_action::UiAction;

/// Represents every possible app event
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    RequestedAction(UiAction),
    EditorOpened,
    CouldNotOpenEditor,
    CompilationStart,
    CompilationEnd(bool),
    CompilationOutputLine(String),
    FileSaved(PathBuf),
    OutputCheckPassed(usize),
    OutputCheckFailed(usize, Diff),
    RunStart(usize),
    RunEnd(usize),
    RunOutputLine(usize, String),
    RunFail(usize, String),
}
