use std::path::PathBuf;

use log::error;

use crate::{
    core::parser::from_dir::FromDir,
    models::{exo::Exo, ui_action::UiAction},
};

use super::app::App;

/// Functions related to handling actions requested by the UI
impl App {
    pub(super) fn on_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::StartExo { exo_folder } => self.start_exo_request(exo_folder),
            UiAction::StopExo => self.stop_exo_request(),
            UiAction::StopApp => self.stop_app(),
        }
    }

    pub(super) fn start_exo_request(&mut self, exo_folder: PathBuf) {
        let exo = Exo::from_dir(&exo_folder).unwrap().0; // todo fix unwrap
        match App::start_exo(&self.work_handler, &exo) {
            Ok(cr) => {
                self.current_run = Some(cr);
            }
            Err(err) =>
            //TODO send this to the ui
            {
                error!("Could not launch exo {}", err);
            }
        }
    }

    pub(super) fn stop_exo_request(&mut self) {
        if let Ok(mut wh) = self.work_handler.lock() {
            wh.stop_all_workers_and_wait();
        }
    }

    pub(super) fn stop_app(&mut self) {
        self.stop_exo_request();
        self.run = false; // so the run_forever() while stops
    }
}
