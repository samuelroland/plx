mod commands;
use commands::{
    courses::{clone_course, get_local_courses, open_course},
    sessions::{get_sessions, join_session, start_session},
};
use std::sync::Mutex;

use plx::{
    live::{
        client::LiveClient,
        config::LiveConfig,
        server::{
            DEFAULT_LIVE_PORT, PROTOCOL_VERSION, QUERYSTRING_LIVE_CLIENT_ID_FIELD,
            QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD,
        },
    },
    models::project::Project,
};
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

struct Current {
    config: LiveConfig,
    project: Project,
}

struct AppData {
    client: Mutex<Option<LiveClient>>,
    current: Mutex<Option<Current>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            get_local_courses,
            clone_course,
            open_course,
            get_sessions,
            start_session,
            join_session,
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
            open_course,
            get_sessions,
            start_session,
            join_session
        ])
        .setup(|app| {
            app.manage(AppData {
                client: Mutex::new(None),
                current: Mutex::new(None),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
