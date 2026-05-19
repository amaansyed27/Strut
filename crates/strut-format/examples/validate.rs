use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: cargo run -p strut-format --example validate -- <file.strut>")?;

    let package = strut_format::read_strut_file(&path)?;

    println!("valid: {}", path.display());
    println!("format: {}", package.manifest.schema_version);
    println!("document: {}", package.document.name);
    println!("artboards: {}", package.document.artboards.len());
    println!("timelines: {}", package.document.timelines.len());
    println!("state machines: {}", package.document.state_machines.len());
    println!("bindings: {}", package.document.bindings.len());
    println!("events: {}", package.document.events.len());

    Ok(())
}
