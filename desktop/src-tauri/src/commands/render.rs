use dme_core::{
    detect_lang_from_file_extension, get_default_theme_css, highlight_code,
    markdown_content_to_highlighted_html,
};

// Any kind of rendering needed by the frontend, rendered in backend for performance or quality reasons

#[tauri::command]
#[specta::specta]
pub async fn highlight_code_with_tree_sitter(file: String, code: String) -> Result<String, String> {
    let lang = &detect_lang_from_file_extension(&file);
    highlight_code(lang, &code).map(|c| c.to_safe_html_string())
}

#[tauri::command]
#[specta::specta]
pub async fn render_markdown_with_highlighting(content: String) -> Result<String, String> {
    markdown_content_to_highlighted_html(&content).map(|h| h.to_safe_html_string())
}

#[tauri::command]
#[specta::specta]
pub async fn load_default_theme_css() -> String {
    get_default_theme_css()
}
