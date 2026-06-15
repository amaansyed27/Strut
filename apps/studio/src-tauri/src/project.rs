use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use crate::*;
use std::fs;
pub fn default_projects_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        return PathBuf::from(home).join("Documents").join("Strut Projects");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("Strut Projects")
}

pub fn sanitize_project_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("project name is required".to_string());
    }

    let sanitized: String = trimmed
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_')
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if sanitized.is_empty() {
        Err("project name needs letters or numbers".to_string())
    } else {
        Ok(sanitized)
    }
}

pub fn sanitize_animation_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("animation name is required".to_string());
    }
    let sanitized = trimmed
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, ' ' | '-' | '_' | ':' | '(' | ')' | '/')
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if sanitized.is_empty() {
        Err("animation name needs letters or numbers".to_string())
    } else {
        Ok(sanitized)
    }
}

pub fn project_info(name: String, path: PathBuf) -> ProjectInfo {
    ProjectInfo {
        name,
        path: path.display().to_string(),
        files: vec![
            ProjectFile {
                name: PROJECT_MANIFEST_FILE.to_string(),
                path: path.join(PROJECT_MANIFEST_FILE).display().to_string(),
                kind: "project".to_string(),
            },
            ProjectFile {
                name: "main.strut".to_string(),
                path: path.join(MAIN_SCENE_FILE).display().to_string(),
                kind: "scene".to_string(),
            },
            ProjectFile {
                name: "animations".to_string(),
                path: path.join(ANIMATION_SCENE_DIR).display().to_string(),
                kind: "folder".to_string(),
            },
            ProjectFile {
                name: "operation-batches.json".to_string(),
                path: path.join(OPERATION_BATCHES_FILE).display().to_string(),
                kind: "operations".to_string(),
            },
            ProjectFile {
                name: "assets".to_string(),
                path: path.join("assets").display().to_string(),
                kind: "folder".to_string(),
            },
            ProjectFile {
                name: "exports".to_string(),
                path: path.join("exports").display().to_string(),
                kind: "folder".to_string(),
            },
        ],
    }
}

pub fn project_manifest_value(name: &str, timestamp: u64) -> Value {
    project_manifest_value_with_animations(name, timestamp, Vec::new())
}

pub fn project_manifest_value_with_animations(
    name: &str,
    timestamp: u64,
    animations: Vec<Value>,
) -> Value {
    json!({
        "name": name,
        "createdAt": timestamp,
        "updatedAt": timestamp,
        "format": "0.2.0",
        "mainScene": MAIN_SCENE_FILE,
        "operationBatches": OPERATION_BATCHES_FILE,
        "studioState": STUDIO_STATE_FILE,
        "animations": animations
    })
}

pub fn read_project_manifest(root: &Path) -> Result<Value, String> {
    let manifest_path = root.join(PROJECT_MANIFEST_FILE);
    if !manifest_path.exists() {
        return Ok(json!({}));
    }
    let raw = fs::read_to_string(&manifest_path).map_err(|error| error.to_string())?;
    serde_json::from_str::<Value>(&raw).map_err(|error| error.to_string())
}

pub fn project_name_from_manifest(manifest: &Value, root: &Path) -> String {
    manifest
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| root.file_name().map(|name| name.to_string_lossy().to_string()))
        .unwrap_or_else(|| "Strut Project".to_string())
}

pub fn project_animation_manifest_entry(animation: &ProjectAnimationRecord) -> Value {
    json!({
        "id": animation.id,
        "name": animation.name,
        "chatId": animation.chat_id,
        "scene": animation.scene,
        "operationBatches": project_animation_operation_path(animation),
        "studioState": null,
        "updatedAt": animation.updated_at
    })
}

pub fn project_animation_operation_path(animation: &ProjectAnimationRecord) -> Option<String> {
    Some(format!("{ANIMATION_OPERATION_DIR}/{}.json", animation.id))
}

pub fn write_project_manifest_with_animation_records(
    root: &Path,
    name: &str,
    animations: &[ProjectAnimationRecord],
) -> Result<(), String> {
    let entries = animations
        .iter()
        .map(project_animation_manifest_entry)
        .collect::<Vec<_>>();
    fs::write(
        root.join(PROJECT_MANIFEST_FILE),
        serde_json::to_string_pretty(&project_manifest_value_with_animations(
            name,
            unix_timestamp(),
            entries,
        ))
        .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

pub fn read_project_animation_records(root: &Path) -> Result<Vec<ProjectAnimationRecord>, String> {
    let manifest = read_project_manifest(root)?;
    let Some(entries) = manifest.get("animations").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut records = Vec::new();
    for entry in entries {
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "project animation entry needs id".to_string())?
            .to_string();
        let name = entry
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Untitled animation")
            .to_string();
        let chat_id = entry
            .get("chatId")
            .and_then(Value::as_str)
            .map(str::to_string);
        let scene = entry
            .get("scene")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("project animation '{id}' needs scene"))?
            .to_string();
        let document = read_project_document(root, &scene)?;
        let operation_batches = entry
            .get("operationBatches")
            .and_then(Value::as_str)
            .map(|path| read_operation_batches_from(root, path))
            .transpose()?
            .unwrap_or_default();
        validate_operation_batches(&operation_batches, &document)?;
        let updated_at = entry
            .get("updatedAt")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        records.push(ProjectAnimationRecord {
            id,
            name,
            chat_id,
            scene,
            operation_batches,
            selection: None,
            document,
            updated_at,
        });
    }
    Ok(records)
}

pub fn ensure_project_root(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("project path is required".to_string());
    }
    let root = PathBuf::from(trimmed);
    if root.exists() && !root.is_dir() {
        return Err(format!("Project path is not a folder: {}", root.display()));
    }
    Ok(root)
}

pub fn safe_project_file_path(
    root: &Path,
    relative_path: &str,
    label: &str,
) -> Result<PathBuf, String> {
    let trimmed = relative_path.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} path is required"));
    }

    let candidate = Path::new(trimmed);
    if candidate.is_absolute() {
        return Err(format!("{label} path must be relative to the project root"));
    }

    for component in candidate.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(format!(
                "{label} path must stay inside the project root: {trimmed}"
            ));
        }
    }

    let resolved = root.join(candidate);
    if resolved.exists() {
        let canonical_root = root.canonicalize().map_err(|error| {
            format!(
                "Could not canonicalize project root {}: {error}",
                root.display()
            )
        })?;
        let canonical_resolved = resolved.canonicalize().map_err(|error| {
            format!(
                "Could not canonicalize {label} path {}: {error}",
                resolved.display()
            )
        })?;
        if !canonical_resolved.starts_with(&canonical_root) {
            return Err(format!(
                "{label} path must stay inside the project root: {trimmed}"
            ));
        }
    }

    Ok(resolved)
}

pub fn read_project_document(root: &Path, main_scene: &str) -> Result<strut_core::Document, String> {
    let scene_path = safe_project_file_path(root, main_scene, "mainScene")?;
    if scene_path.exists()
        && scene_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("strut"))
            .unwrap_or(false)
    {
        return strut_format::read_strut_file(&scene_path)
            .map(|package| package.document)
            .map_err(|error| error.to_string());
    }

    if scene_path.exists() {
        return read_legacy_document_json(&scene_path);
    }

    let legacy_path = root.join(LEGACY_STARTER_SCENE_FILE);
    if legacy_path.exists() {
        return read_legacy_document_json(&legacy_path);
    }

    Err(format!(
        "No Strut scene found. Expected {} or {}",
        root.join(MAIN_SCENE_FILE).display(),
        legacy_path.display()
    ))
}

pub fn read_legacy_document_json(path: &Path) -> Result<strut_core::Document, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let document =
        serde_json::from_str::<strut_core::Document>(&raw).map_err(|error| error.to_string())?;
    strut_format::validate_document(&document).map_err(|error| error.to_string())?;
    Ok(document)
}

pub fn read_operation_batches(root: &Path) -> Result<Vec<OperationBatchRecord>, String> {
    read_operation_batches_from(root, OPERATION_BATCHES_FILE)
}

pub fn read_operation_batches_from(
    root: &Path,
    relative_path: &str,
) -> Result<Vec<OperationBatchRecord>, String> {
    let path = safe_project_file_path(root, relative_path, "operation batches")?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str::<Vec<OperationBatchRecord>>(&raw).map_err(|error| error.to_string())
}

pub fn read_selection_state(root: &Path) -> Result<Option<PersistedSelectionState>, String> {
    let path = root.join(STUDIO_STATE_FILE);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str::<PersistedSelectionState>(&raw)
        .map(Some)
        .map_err(|error| error.to_string())
}

