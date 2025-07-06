use std::ops::Deref;

use plx::models::ui_action::UiAction;
use tauri::{AppHandle, Manager};

use crate::AppData;

#[tauri::command]
#[specta::specta]
pub async fn send_ui_action_to_app(app: AppHandle, action: UiAction) {
    let state = app.state::<AppData>();
    if let Some(guard) = state.ui_action_tx.lock().unwrap().deref() {
        guard.send(action);
    };
}
