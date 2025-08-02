// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::File;

use simplelog::*;

fn main() {
    // Generate a simple debug file to be able to send logs on Windows where println! are not visible
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("plx-desktop-debug.log").expect("Failed to create log file"),
    )
    .expect("Failed to initialize WriteLogger");

    plx_lib::run()
}
