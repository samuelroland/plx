mod commands;
use std::sync::{mpsc::Sender, Mutex};

use commands::{
    courses::{clone_course, get_local_courses, git_pull_all_courses, load_full_course_details},
    render::{
        highlight_code_with_tree_sitter, load_default_theme_css, render_markdown_with_highlighting,
    },
    train::send_ui_action_to_app,
};

use plx_core::{
    live::server::{
        DEFAULT_LIVE_PORT, PROTOCOL_VERSION, QUERYSTRING_LIVE_CLIENT_ID_FIELD,
        QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD,
    },
    models::ui_action::UiAction,
};
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

struct AppData {
    /// We start an App instance and we start run_forever() in a separated thread
    /// we will have a way to send UiAction here to keep a link with this instance
    ui_action_tx: Mutex<Option<Sender<UiAction>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            get_local_courses,
            clone_course,
            highlight_code_with_tree_sitter,
            load_default_theme_css,
            render_markdown_with_highlighting,
            send_ui_action_to_app,
            git_pull_all_courses
        ])
        .constant("PROTOCOL_VERSION", PROTOCOL_VERSION)
        .constant("DEFAULT_LIVE_PORT", DEFAULT_LIVE_PORT)
        .constant(
            "QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD",
            QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD,
        )
        .constant(
            "QUERYSTRING_LIVE_CLIENT_ID_FIELD",
            QUERYSTRING_LIVE_CLIENT_ID_FIELD,
        );

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/ts/commands.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_local_courses,
            clone_course,
            highlight_code_with_tree_sitter,
            load_default_theme_css,
            load_full_course_details,
            render_markdown_with_highlighting,
            send_ui_action_to_app,
            git_pull_all_courses
        ])
        .setup(|app| {
            app.manage(AppData {
                ui_action_tx: Mutex::new(None),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
