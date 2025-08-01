use std::{fs::read_to_string, path::PathBuf};

use log::error;

use super::{app::App, exo_status_report::FileContent};

/// Functions related to handling file saving events
impl App {
    /// File saved event handler
    /// Called when one the current exo files gets saved
    pub(super) fn on_file_save(&mut self, path: PathBuf) {
        if let Some(ref mut cr) = self.current_run {
            let base = &cr.exo.folder;
            let maybe_file = cr
                .edited_files_content
                .iter_mut()
                .find(|e| base.join(&e.path) == path);
            let new_file_content = read_to_string(&path).unwrap_or_default();
            match maybe_file {
                Some(file) => file.content = new_file_content,
                None => cr.edited_files_content.push(FileContent {
                    path: path
                        .strip_prefix(base)
                        .unwrap()
                        .to_str()
                        .unwrap_or_default()
                        .to_string(),
                    content: new_file_content,
                }),
            }
            let compile = App::compile(&self.work_handler, &cr.exo);
            if let Err(err) = compile {
                error!("Error Starting Compilation {}", err);
            }
        }
    }
}
