use super::app::App;

/// Functions related to handling compilation events
impl App {
    /// Compilation output event handler
    /// Gets called when the compilation process outputs a new line
    pub(super) fn on_compilation_start(&mut self) {
        if let Some(ref mut cr) = self.current_run {
            cr.compilation_output.clear();
            cr.compilation_running = true;
            self.send_new_exo_status();
        }
    }
    pub(super) fn on_compilation_output(&mut self, line: String) {
        if let Some(ref mut cr) = self.current_run {
            cr.compilation_output.push(line);
            cr.compilation_running = true;
            self.send_new_exo_status();
        }
    }

    /// Compilation finished event handler
    /// Gets called when the target binary compilation ends
    pub(super) fn on_compilation_end(&mut self, success: bool) {
        if let Some(ref mut cr) = self.current_run {
            cr.compilation_running = false;
            cr.compilation_success = success;
            if success {
                self.start_runners();
            }
        }
        self.send_new_exo_status();
    }
}
