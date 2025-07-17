use dme_core::util::git::GitRepos;
use log::info;
use plx::{
    app::{app::App, exo_status_report::ExoStatusReport},
    core::{file_utils::file_utils::list_dir_folders, parser::from_dir::FromDir},
    live::config::LiveConfig,
    models::course::Course,
};
use std::{fs::create_dir_all, path::PathBuf, sync::mpsc, thread};
use tauri::{ipc::Channel, AppHandle, Manager};

use etcetera::{AppStrategy, AppStrategyArgs};
use serde::Serialize;
use specta::Type;

use crate::AppData;

#[derive(Serialize, Debug, Type)]
pub struct CourseInfo {
    name: String,
    folder: PathBuf,
    config: Option<LiveConfig>,
}

fn get_base_directory() -> PathBuf {
    let folder = etcetera::choose_app_strategy(AppStrategyArgs {
        app_name: "plx".to_string(),
        ..Default::default()
    })
    .unwrap()
    .data_dir()
    .to_path_buf();
    info!("Using base directory: {folder:?}");
    if !folder.exists() {
        info!("Created non existant base directory {folder:?}");
        create_dir_all(&folder).expect("Couldn't create directory {folder}");
    }
    folder
}

#[tauri::command]
#[specta::specta]
pub async fn get_local_courses() -> Vec<CourseInfo> {
    let base = get_base_directory();
    list_dir_folders(&base)
        .unwrap()
        .iter()
        .filter_map(|f| Course::from_dir(f).ok())
        .map(|(p, _)| {
            let config = LiveConfig::from_course(&p)
                .map_err(|e| {
                    let message = format!("{e}");
                    println!("Error {message}");
                    message
                })
                .ok();
            CourseInfo {
                name: p.name.clone(),
                folder: p.get_folder(),
                config,
            }
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub async fn clone_course(repos: String) -> bool {
    let base = get_base_directory();
    GitRepos::from_clone(&repos, &base, Some(1), true).is_ok()
}

#[tauri::command]
#[specta::specta]
pub async fn load_full_course_details(
    app: AppHandle,
    course_path: PathBuf,
    exo_status_ui_channel: Channel<ExoStatusReport>,
) -> Result<Course, String> {
    let state = app.state::<AppData>();
    let (exo_status_tx, exo_status_rx) = mpsc::channel();
    let (ui_action_tx, ui_action_rx) = mpsc::channel();
    let course = Course::from_dir(&course_path)
        .map_err(|(e, _)| e.to_string())?
        .0;

    // TODO: fix this unwrap mess

    // Setup app instance
    let app = App::new_in_folder(&course_path, exo_status_tx, ui_action_rx)
        .map_err(|e| e.to_string())
        .unwrap();

    // Run it forever in another thread, we move it so we lose it's reference, but the
    // 2 channels are enough to communicate with it
    thread::spawn(move || {
        app.run_forever();
    });

    // Just save the ui_action_tx to be reused by send_ui_action_to_app
    *state.ui_action_tx.lock().unwrap() = Some(ui_action_tx);

    // Listen on exo status update sent by the App and just forward that in the Tauri's channel
    thread::spawn(move || {
        while let Ok(status) = exo_status_rx.recv() {
            exo_status_ui_channel.send(status).unwrap();
        }
    });

    Ok(course)
}
