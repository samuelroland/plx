use core::app::App;

pub mod core;
pub mod models;
pub mod ui;

fn main() {
    match App::new() {
        Ok(app) => app.run_forever(),
        Err(err) => {
            eprintln!("Error starting plx {err}");
        }
    }
}
