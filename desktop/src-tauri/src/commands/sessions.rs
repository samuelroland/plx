use std::ops::DerefMut;

use plx::live::{client::LiveClient, server::DEFAULT_LIVE_PORT, session::Session};
use tauri::{AppHandle, Manager};

use crate::AppData;

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

#[tauri::command]
#[specta::specta]
pub fn get_sessions(app: AppHandle) -> Result<Vec<Session>, String> {
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
pub fn start_session(app: AppHandle, name: String) -> Result<Session, String> {
    let state = app.state::<AppData>();
    init_client_if_none(state.client.lock().unwrap().deref_mut());
    let mut guard = state.client.lock().unwrap();
    if let Some(cli) = guard.deref_mut() {
        if let Some(current) = state.current.lock().unwrap().as_ref() {
            let group_id = current.config.group_id.clone();
            let session = cli.start_session(&name, &group_id)?;
            Ok(session)
        } else {
            Err("No opened course".to_string())
        }
    } else {
        Err("No live client".to_string())
    }
}

#[tauri::command]
#[specta::specta]
pub fn join_session(app: AppHandle, session: Session) -> Result<Session, String> {
    let state = app.state::<AppData>();
    init_client_if_none(state.client.lock().unwrap().deref_mut());
    let mut guard = state.client.lock().unwrap();
    if let Some(cli) = guard.deref_mut() {
        let session = cli.join_session(&session.name, &session.group_id)?;
        Ok(session)
    } else {
        Err("No live client".to_string())
    }
}

// #[tauri::command]
// #[specta::specta]
// pub async fn setup_answers_streaming(
//     app: AppHandle,
//     on_event: Channel<Answer>,
// ) -> Result<Session, String> {
//     let state = app.state::<AppData>();
//     init_client_if_none(state.client.lock().unwrap().deref_mut());
//     let mut guard = state.client.lock().unwrap();
//     if let Some(cli) = guard.deref_mut() {
//         cli.wait_all_next_events();
//         Ok(session)
//     } else {
//         Err("No live client".to_string())
//     }
// }
//
