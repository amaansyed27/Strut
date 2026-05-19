#[tauri::command]
fn studio_status() -> strut_format::StudioStatus {
    let document = strut_core::Document::sample_login_button();
    strut_format::StudioStatus::from_document("Strut Studio", &document)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![studio_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
