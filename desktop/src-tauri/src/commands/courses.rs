use dme_core::util::git::GitRepos;
use plx::{
    core::{file_utils::file_utils::list_dir_folders, parser::from_dir::FromDir},
    live::config::LiveConfig,
    models::project::Project,
};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use etcetera::{AppStrategy, AppStrategyArgs};
use serde::Serialize;
use specta::Type;

use crate::{AppData, Current};
#[derive(Serialize, Debug, Clone, Type)]
pub struct CourseInfo {
    name: String,
    folder: PathBuf,
}

fn get_base_directory() -> PathBuf {
    etcetera::choose_app_strategy(AppStrategyArgs {
        app_name: "plx".to_string(),
        ..Default::default()
    })
    .unwrap()
    .data_dir()
    .to_path_buf()
}

#[tauri::command]
#[specta::specta]
pub async fn get_local_courses() -> Vec<CourseInfo> {
    let base = get_base_directory();
    list_dir_folders(&base)
        .unwrap()
        .iter()
        .filter_map(|f| Project::from_dir(f).ok())
        .map(|(p, _)| CourseInfo {
            name: p.name.clone(),
            folder: p.get_folder(),
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
pub async fn open_course(app: AppHandle, path: String) -> Result<CourseInfo, String> {
    let state = app.state::<AppData>();
    let (p, _) = Project::from_dir(&PathBuf::from(&path)).map_err(|(e, _)| format!("{}", e))?;
    let config = LiveConfig::from_course(&p).map_err(|e| format!("{}", e))?;
    let course_info = CourseInfo {
        name: p.name.clone(),
        folder: PathBuf::from(path),
    };
    *state.current.lock().unwrap() = Some(Current { project: p, config });
    Ok(course_info)
}
