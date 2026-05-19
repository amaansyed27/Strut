#[tauri::command]
fn studio_status() -> strut_format::StudioStatus {
    let sample_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../samples/login-button.strut");

    match strut_format::read_strut_file(&sample_path) {
        Ok(package) => strut_format::StudioStatus::from_document_with_source(
            "Strut Studio",
            &package.document,
            sample_path.display().to_string(),
        ),
        Err(_) => {
            let document = strut_core::Document::sample_login_button();
            strut_format::StudioStatus::from_document("Strut Studio", &document)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![studio_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
