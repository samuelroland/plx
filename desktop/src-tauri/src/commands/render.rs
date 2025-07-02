use dme_core::{detect_lang_from_file_extension, highlight_code, preview::preview::Html};

// Any kind of rendering needed by the frontend, rendered in backend for performance or quality reasons

#[tauri::command]
#[specta::specta]
pub async fn highlight_code_with_tree_sitter(file: String, code: String) -> Result<Html, String> {
    highlight_code(&detect_lang_from_file_extension(&file), &code)
}
