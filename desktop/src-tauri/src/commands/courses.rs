use dme_core::util::git::GitRepos;
use log::{error, info, warn};
use plx_core::{
    app::{app::App, exo_status_report::ExoStatusReport},
    core::{file_utils::file_utils::list_dir_folders, parser::from_dir::FromDir},
    dy::error::ParseError,
    live::config::LiveConfig,
    models::course::Course,
};
use std::{fs::create_dir_all, path::PathBuf, sync::mpsc, thread, vec};
use tauri::{ipc::Channel, AppHandle, Manager};

use etcetera::{AppStrategy, AppStrategyArgs};
use serde::Serialize;
use specta::Type;

use crate::AppData;

#[derive(Serialize, Debug, Type)]
pub struct CourseWithConfig {
    course: Course,
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
pub async fn get_local_courses() -> Vec<CourseWithConfig> {
    let base = get_base_directory();
    match list_dir_folders(&base) {
        Ok(folders) => folders
            .iter()
            .filter_map(|f| match Course::from_dir(f, false) {
                Ok(course) => Some(course),
                Err(e) => {
                    log::warn!("{e}");
                    None
                }
            })
            .map(|(_, course)| {
                let config = LiveConfig::from_course(&course)
                    .map_err(|e| {
                        let message = format!("{e}");
                        println!("Error {message}");
                        message
                    })
                    .ok();
                CourseWithConfig { course, config }
            })
            .collect(),
        Err(e) => {
            log::error!("{e}");
            vec![]
        }
    }
}

#[tauri::command]
#[specta::specta]
/// Clone a given course repository inside the global courses folder
/// and return the error in case it failed
pub async fn clone_course(repos: String) -> Result<(), String> {
    let base = get_base_directory();
    GitRepos::from_clone(&repos, &base, Some(1), true)?;
    Ok(())
}

#[derive(Serialize, Debug)]
#[typeshare::typeshare]
pub struct CourseWithErrors {
    course: Course,
    errors: Vec<ParseError>,
}

#[tauri::command]
// Note: do not use #[specta::specta] here, write the TS command by hand
pub async fn load_full_course_details(
    app: AppHandle,
    course_path: PathBuf,
    exo_status_ui_channel: Channel<ExoStatusReport>,
) -> Result<CourseWithErrors, String> {
    let state = app.state::<AppData>();
    let (exo_status_tx, exo_status_rx) = mpsc::channel();
    let (ui_action_tx, ui_action_rx) = mpsc::channel();
    let (errors, course) = Course::from_dir(&course_path, true) // deep mode this time
        .map_err(|e| e.to_string())?;

    // Setup app instance
    let app =
        App::new_in_folder(&course_path, exo_status_tx, ui_action_rx).map_err(|e| e.to_string())?;

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
            let _ = exo_status_ui_channel
                .send(status)
                .map_err(|e| error!("{e}"));
        }
    });

    Ok(CourseWithErrors { course, errors })
}

#[tauri::command]
#[specta::specta]
pub async fn git_pull_all_courses() {
    let base = get_base_directory();
    match list_dir_folders(&base) {
        Ok(folders) => {
            folders.iter().for_each(|r| {
                match GitRepos::from_existing_folder(&base.join(r)) {
                    Ok(repos) => {
                        if let Err(e) = repos.pull() {
                            warn!("Could not pull from repository {r:?}: {e:?}");
                        }
                    }
                    Err(e) => warn!("Folder {r:?} is not a Git repository: {e:?}"),
                };
            });
        }
        Err(e) => error!("Could not get the list of folders in base directory: {e}"),
    }
}
