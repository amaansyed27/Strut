use serde::{Deserialize, Serialize};
use strut_core::Document;
use thiserror::Error;

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
    pub sample_artboards: usize,
    pub sample_state_machines: usize,
}

#[derive(Debug, Error, PartialEq)]
pub enum FormatError {
    #[error("manifest format must be 'strut'")]
    InvalidFormat,
    #[error("schema version must start with 0.1")]
    UnsupportedSchemaVersion,
    #[error("document must contain at least one artboard")]
    MissingArtboard,
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

impl StudioStatus {
    pub fn from_document(app: impl Into<String>, document: &Document) -> Self {
        Self {
            app: app.into(),
            core_version: CORE_VERSION.to_string(),
            format_version: FORMAT_VERSION.to_string(),
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

    Ok(())
}

pub fn validate_document(document: &Document) -> Result<(), FormatError> {
    if document.artboards.is_empty() {
        return Err(FormatError::MissingArtboard);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_manifest_validates() {
        assert_eq!(validate_manifest(&Manifest::current()), Ok(()));
    }

    #[test]
    fn sample_document_validates() {
        let document = Document::sample_login_button();

        assert_eq!(validate_document(&document), Ok(()));
    }
}
