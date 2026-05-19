#[tauri::command]
fn studio_status() -> strut_format::StudioStatus {
    let sample_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../samples/minimal-bot.strut");

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

#[tauri::command]
fn sample_document() -> strut_core::Document {
    let sample_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../samples/minimal-bot.strut");

    strut_format::read_strut_file(&sample_path)
        .map(|package| package.document)
        .unwrap_or_else(|_| strut_core::Document::sample_minimal_bot())
}

#[tauri::command]
fn generate_character(prompt: String) -> strut_core::Document {
    let spec = strut_core::character_spec_from_prompt(&prompt);
    strut_core::Document::generate_character(spec)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            studio_status,
            sample_document,
            generate_character
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
