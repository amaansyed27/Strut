use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Component, Path};
use strut_core::Document;
use strut_core::Node;
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub const FORMAT_VERSION: &str = "format 0.1.0";
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub format: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub document: String,
    #[serde(rename = "createdBy")]
    pub created_by: String,
    #[serde(rename = "minimumRuntime")]
    pub minimum_runtime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StudioStatus {
    pub app: String,
    pub core_version: String,
    pub format_version: String,
    pub sample_name: String,
    pub sample_source: String,
    pub sample_artboards: usize,
    pub sample_state_machines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrutPackage {
    pub manifest: Manifest,
    pub document: Document,
}

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("manifest format must be 'strut'")]
    InvalidFormat,
    #[error("schema version must start with 0.1")]
    UnsupportedSchemaVersion,
    #[error("manifest document path is unsafe")]
    UnsafeDocumentPath,
    #[error("document must contain at least one artboard")]
    MissingArtboard,
    #[error("artboard '{0}' dimensions must be greater than zero")]
    InvalidArtboardSize(String),
    #[error("node id '{0}' appears more than once")]
    DuplicateNodeId(String),
    #[error("state machine '{0}' must contain at least one state")]
    MissingState(String),
    #[error("state machine '{machine}' has duplicate input '{input}'")]
    DuplicateInput { machine: String, input: String },
    #[error("timeline '{0}' duration must be greater than zero")]
    InvalidTimelineDuration(String),
    #[error("timeline '{timeline}' targets unknown node '{target}'")]
    MissingTimelineTarget { timeline: String, target: String },
    #[error("transition in '{machine}' references unknown state '{state}'")]
    UnknownTransitionState { machine: String, state: String },
    #[error("transition in '{machine}' references unknown timeline '{timeline}'")]
    UnknownTransitionTimeline { machine: String, timeline: String },
    #[error("zip entry '{0}' is missing")]
    MissingZipEntry(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Manifest {
    pub fn current() -> Self {
        Self {
            format: "strut".to_string(),
            schema_version: "0.1.0".to_string(),
            document: "document.json".to_string(),
            created_by: "strut-studio".to_string(),
            minimum_runtime: "0.1.0".to_string(),
        }
    }
}

impl StrutPackage {
    pub fn current(document: Document) -> Self {
        Self {
            manifest: Manifest::current(),
            document,
        }
    }
}

impl StudioStatus {
    pub fn from_document(app: impl Into<String>, document: &Document) -> Self {
        Self::from_document_with_source(app, document, "generated sample")
    }

    pub fn from_document_with_source(
        app: impl Into<String>,
        document: &Document,
        source: impl Into<String>,
    ) -> Self {
        Self {
            app: app.into(),
            core_version: CORE_VERSION.to_string(),
            format_version: FORMAT_VERSION.to_string(),
            sample_name: document.name.clone(),
            sample_source: source.into(),
            sample_artboards: document.artboards.len(),
            sample_state_machines: document.state_machines.len(),
        }
    }
}

pub fn validate_manifest(manifest: &Manifest) -> Result<(), FormatError> {
    if manifest.format != "strut" {
        return Err(FormatError::InvalidFormat);
    }

    if !manifest.schema_version.starts_with("0.1") {
        return Err(FormatError::UnsupportedSchemaVersion);
    }

    validate_document_path(&manifest.document)?;

    Ok(())
}

pub fn validate_document(document: &Document) -> Result<(), FormatError> {
    if document.artboards.is_empty() {
        return Err(FormatError::MissingArtboard);
    }

    let mut node_ids = HashSet::new();
    for artboard in &document.artboards {
        if artboard.width <= 0.0 || artboard.height <= 0.0 {
            return Err(FormatError::InvalidArtboardSize(artboard.name.clone()));
        }

        for node in flatten_nodes(&artboard.nodes) {
            if !node_ids.insert(node.id) {
                return Err(FormatError::DuplicateNodeId(node.id.to_string()));
            }
        }
    }

    let timeline_names: HashMap<_, _> = document
        .timelines
        .iter()
        .map(|timeline| (timeline.name.as_str(), timeline))
        .collect();

    for timeline in &document.timelines {
        if timeline.duration_ms == 0 {
            return Err(FormatError::InvalidTimelineDuration(timeline.name.clone()));
        }

        for track in &timeline.tracks {
            if !node_ids.contains(&track.target) {
                return Err(FormatError::MissingTimelineTarget {
                    timeline: timeline.name.clone(),
                    target: track.target.to_string(),
                });
            }
        }
    }

    for machine in &document.state_machines {
        if machine.states.is_empty() {
            return Err(FormatError::MissingState(machine.name.clone()));
        }

        let mut seen = HashSet::new();
        for input in &machine.inputs {
            if !seen.insert(input.name.as_str()) {
                return Err(FormatError::DuplicateInput {
                    machine: machine.name.clone(),
                    input: input.name.clone(),
                });
            }
        }

        let states: HashSet<_> = machine.states.iter().map(String::as_str).collect();
        for transition in &machine.transitions {
            for state in [&transition.from, &transition.to] {
                if !states.contains(state.as_str()) {
                    return Err(FormatError::UnknownTransitionState {
                        machine: machine.name.clone(),
                        state: state.clone(),
                    });
                }
            }

            if !timeline_names.contains_key(transition.timeline.as_str()) {
                return Err(FormatError::UnknownTransitionTimeline {
                    machine: machine.name.clone(),
                    timeline: transition.timeline.clone(),
                });
            }
        }
    }

    Ok(())
}

pub fn validate_package(package: &StrutPackage) -> Result<(), FormatError> {
    validate_manifest(&package.manifest)?;
    validate_document(&package.document)?;
    Ok(())
}

pub fn read_strut_file(path: impl AsRef<Path>) -> Result<StrutPackage, FormatError> {
    let file = File::open(path)?;
    read_strut_reader(file)
}

pub fn read_strut_reader<R>(reader: R) -> Result<StrutPackage, FormatError>
where
    R: Read + Seek,
{
    let mut archive = ZipArchive::new(reader)?;
    let manifest: Manifest = read_json_entry(&mut archive, "manifest.json")?;
    validate_manifest(&manifest)?;

    let document_path = manifest.document.clone();
    let document: Document = read_json_entry(&mut archive, &document_path)?;
    let package = StrutPackage { manifest, document };
    validate_package(&package)?;
    Ok(package)
}

pub fn write_strut_file(path: impl AsRef<Path>, package: &StrutPackage) -> Result<(), FormatError> {
    validate_package(package)?;
    let file = File::create(path)?;
    write_strut_writer(file, package)
}

pub fn write_strut_writer<W>(writer: W, package: &StrutPackage) -> Result<(), FormatError>
where
    W: Write + Seek,
{
    validate_package(package)?;
    let mut archive = ZipWriter::new(writer);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    archive.start_file("manifest.json", options)?;
    serde_json::to_writer_pretty(&mut archive, &package.manifest)?;

    archive.start_file(&package.manifest.document, options)?;
    serde_json::to_writer_pretty(&mut archive, &package.document)?;

    archive.add_directory("assets/", options)?;
    archive.add_directory("previews/", options)?;
    archive.finish()?;
    Ok(())
}

fn read_json_entry<T, R>(archive: &mut ZipArchive<R>, path: &str) -> Result<T, FormatError>
where
    T: for<'de> Deserialize<'de>,
    R: Read + Seek,
{
    let mut file = archive
        .by_name(path)
        .map_err(|_| FormatError::MissingZipEntry(path.to_string()))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(serde_json::from_str(&contents)?)
}

fn validate_document_path(path: &str) -> Result<(), FormatError> {
    let path = Path::new(path);
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(FormatError::UnsafeDocumentPath);
    }

    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(FormatError::UnsafeDocumentPath);
        }
    }

    Ok(())
}

fn flatten_nodes(nodes: &[Node]) -> Vec<&Node> {
    let mut flattened = Vec::new();
    for node in nodes {
        flattened.push(node);
        flattened.extend(flatten_nodes(&node.children));
    }
    flattened
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_manifest_validates() {
        assert!(validate_manifest(&Manifest::current()).is_ok());
    }

    #[test]
    fn sample_document_validates() {
        let document = Document::sample_login_button();

        assert!(validate_document(&document).is_ok());
    }

    #[test]
    fn bot_document_validates() {
        let document = Document::sample_minimal_bot();

        assert!(validate_document(&document).is_ok());
    }

    #[test]
    fn package_round_trips_through_zip_container() {
        let package = StrutPackage::current(Document::sample_login_button());
        let mut buffer = std::io::Cursor::new(Vec::new());

        write_strut_writer(&mut buffer, &package).expect("write package");
        buffer.set_position(0);
        let decoded = read_strut_reader(buffer).expect("read package");

        assert_eq!(decoded.manifest, package.manifest);
        assert_eq!(decoded.document.name, "Login Button");
    }

    #[test]
    fn unsafe_document_paths_are_rejected() {
        let mut manifest = Manifest::current();
        manifest.document = "../document.json".to_string();

        assert!(matches!(
            validate_manifest(&manifest),
            Err(FormatError::UnsafeDocumentPath)
        ));
    }
}
