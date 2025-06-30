use dme_core::util::git::GitRepos;
use plx::core::{file_utils::file_utils::list_dir_folders, parser::from_dir::FromDir};
use serde::Serialize;
use std::{ops::DerefMut, path::PathBuf, sync::Mutex};

use etcetera::{AppStrategy, AppStrategyArgs};
use plx::{
    live::{client::LiveClient, config::LiveConfig, server::DEFAULT_LIVE_PORT, session::Session},
    models::project::Project,
};
use specta_typescript::Typescript;
use tauri::{AppHandle, Manager};
use tauri_specta::{collect_commands, Builder};

#[tauri::command]
#[specta::specta]
fn get_sessions(app: AppHandle) -> Result<Vec<Session>, String> {
    let state = app.state::<AppData>();
    init_client_if_none(state.client.lock().unwrap().deref_mut());
    let mut guard = state.client.lock().unwrap();
    if let Some(cli) = guard.deref_mut() {
        if let Some(current) = state.current.lock().unwrap().as_ref() {
            let group_id = current.config.group_id.clone();
            cli.get_sessions(group_id)
        } else {
            Ok(vec![])
        }
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
#[specta::specta]
fn start_session(app: AppHandle, name: String) -> Result<(), String> {
    let state = app.state::<AppData>();
    init_client_if_none(state.client.lock().unwrap().deref_mut());
    let mut guard = state.client.lock().unwrap();
    if let Some(cli) = guard.deref_mut() {
        if let Some(current) = state.current.lock().unwrap().as_ref() {
            let group_id = current.config.group_id.clone();
            cli.start_session(&name, &group_id)?;
            Ok(())
        } else {
            Err("No opened course".to_string())
        }
    } else {
        Err("No live client".to_string())
    }
}

#[tauri::command]
#[specta::specta]
fn join_session(app: AppHandle, session: Session) -> Result<(), String> {
    let state = app.state::<AppData>();
    init_client_if_none(state.client.lock().unwrap().deref_mut());
    let mut guard = state.client.lock().unwrap();
    if let Some(cli) = guard.deref_mut() {
        cli.join_session(&session.name, &session.group_id)?;
        Ok(())
    } else {
        Err("No live client".to_string())
    }
}

fn init_client_if_none(client: &mut Option<LiveClient>) {
    match client {
        Some(_) => {}
        None => {
            let new_client =
                LiveClient::connect("127.0.0.1", DEFAULT_LIVE_PORT, "random ".to_string()).unwrap();
            *client = Some(new_client)
        }
    }
}

use specta::Type;
#[derive(Serialize, Debug, Clone, Type)]
struct ProjectInfo {
    name: String,
    folder: PathBuf,
}
#[tauri::command]
#[specta::specta]
async fn get_projects() -> Vec<ProjectInfo> {
    let base = get_base_directory();
    list_dir_folders(&base)
        .unwrap()
        .iter()
        .filter_map(|f| Project::from_dir(f).ok())
        .map(|(p, _)| ProjectInfo {
            name: p.name.clone(),
            folder: p.get_folder(),
        })
        .collect()
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
async fn clone_project(repos: String) -> bool {
    let base = get_base_directory();
    GitRepos::from_clone(&repos, &base, Some(1), true).is_ok()
}

#[tauri::command]
#[specta::specta]
async fn open_project(app: AppHandle, path: String) -> Result<ProjectInfo, String> {
    let state = app.state::<AppData>();
    let (p, _) = Project::from_dir(&PathBuf::from(&path)).map_err(|(e, _)| format!("{}", e))?;
    let config = LiveConfig::from_course(&p).map_err(|e| format!("{}", e))?;
    let project_info = ProjectInfo {
        name: p.name.clone(),
        folder: PathBuf::from(path),
    };
    *state.current.lock().unwrap() = Some(Current { project: p, config });
    Ok(project_info)
}

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
            get_projects,
            clone_project,
            open_project,
            get_sessions,
            start_session,
            join_session
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_projects,
            clone_project,
            open_project,
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
