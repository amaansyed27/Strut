use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("samples/login-button.strut"));

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let document = match path.file_stem().and_then(|stem| stem.to_str()) {
        Some(stem) if stem.contains("owl") || stem.contains("mascot") => {
            strut_core::Document::sample_owl_mascot()
        }
        Some(stem) if stem.contains("bot") => strut_core::Document::sample_minimal_bot(),
        _ => strut_core::Document::sample_login_button(),
    };
    let package = strut_format::StrutPackage::current(document);
    strut_format::write_strut_file(&path, &package)?;

    println!("wrote {}", path.display());
    Ok(())
}
