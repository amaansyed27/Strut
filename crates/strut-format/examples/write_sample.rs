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

    let package = strut_format::StrutPackage::current(strut_core::Document::sample_login_button());
    strut_format::write_strut_file(&path, &package)?;

    println!("wrote {}", path.display());
    Ok(())
}
