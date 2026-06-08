use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const GENERATION_PLAN_SYSTEM_PROMPT: &str = r##"You convert design prompts into editable Strut motion plans and inspectable Strut operations. Return only JSON in this shape: {"plan": <GenerationPlan>, "operations": <SceneOperation[]>}.

GenerationPlan schema:
- id: short stable string
- name: animation, asset, interaction, object, mascot, logo, loader, or scene name
- subject: {"classification":"dice|logo|loader|mascot|ui|badge|icon|object|scene|abstract|other","label":"human readable subject"}
- parts: 6 to 14 semantic editable parts. Each part has id, name, role, geometry, style, motion_roles, and constraints.
- geometry: {"kind":"rect","x":...,"y":...,"width":...,"height":...,"rx":...}, {"kind":"ellipse","cx":...,"cy":...,"rx":...,"ry":...}, {"kind":"path","d":"SVG path data"}, or {"kind":"text","x":...,"y":...,"value":"...","size":...}
- style: fill, stroke, stroke_width, opacity
- motion_roles: array of role ids such as idle, roll, settle, reveal, sweep, pulse, hover, success, error, loading, transition, custom
- constraints: {"editable":true,"allowed_properties":["fill","stroke","translation.x","translation.y","rotation","scale","opacity"]}
- motion_roles: top-level array of {"id":"settle","purpose":"short purpose","part_refs":["DieBody","TopFace"]}
- states: concise names such as idle, roll, settle, reveal, loading, pulse, success, hover, error
- timelines: named timeline plans with id, name, state, duration_ms, and tracks. Every track target must be a real part id. Keep motion calm and readable.
- editability: {"editable_parts":["..."],"locked_parts":[],"notes":["..."]}

SceneOperation vocabulary:
- create_node, group_nodes, set_property, add_state, add_timeline, add_keyframe, bind_property, emit_event
- Operations must reference part ids from the plan and must be valid before Strut converts them into a document.

Subject rules:
- Rolling dice parts should look like DieBody, FrontFace, TopFace, Pips, EdgeHighlight, SettleShadow.
- Abstract logo parts should look like PrimaryMark, Wordmark, AccentStroke, RevealMask, AnchorGrid.
- Loader parts should look like Track, ActiveSegment, PulseDot, ProgressSweep, Glow.
- Icon or badge parts should look like BadgePlate, InnerShield, SparkGlyph, OrbitStroke, StatusDot.
- Mascot anatomy such as Body, Head, Eyes, Arms, Legs, Face, Smile is allowed only when the user clearly requests a mascot or character.
- Low-energy motion means subtle, calm, breathable motion. It does not imply a face, pet, mascot, body, head, or fixed anatomy.

Return compact JSON; do not explain, do not use markdown, do not return a whole document unless asked to repair an explicit fallback."##;

const CHARACTER_DOCUMENT_SYSTEM_PROMPT: &str = r##"You convert design prompts into editable Strut motion documents. Return only JSON in this shape: {"document": <StrutDocument>}.

StrutDocument schema:
- id: UUID string
- name: animation, asset, interaction, character, or scene name
- artboards: one artboard with id, name, width 960, height 540, and editable nodes
- nodes: recursive objects with id, name, kind, transform, style, shape, children
- kind: group, rect, ellipse, path, text, image, or hit_area
- transform: translate_x, translate_y, rotate, scale_x, scale_y
- style: fill, stroke, stroke_width, opacity, linecap, linejoin
- shape variants use {"type":"rect","x":...,"y":...,"width":...,"height":...,"rx":...}, {"type":"ellipse","cx":...,"cy":...,"rx":...,"ry":...}, {"type":"path","d":"SVG path data"}, {"type":"text","x":...,"y":...,"value":"...","size":...}, or {"type":"none"}
- timelines: include idle_float, wave, blink, scan, and celebrate timelines. Every track target must be a real node id. Keep timelines compact, low-energy, and loop-friendly: one or two tracks per timeline, two or three keyframes per track, 300-1400 ms duration.
- state_machines: include one machine with states idle, float, wave, blink, scan, celebrate, sleep. Treat these as a reusable quiet motion language, not only character moods: idle is stillness, float is gentle breathing/drift, wave is a small acknowledgment, blink is a tiny reset, scan is focused inspection, celebrate is restrained success, sleep is reduced attention.
- bindings and events: arrays, may be empty.

Make the composition, named layers, palette, and motion match the request. Strut can generate logo reveals, SVG animations, icons, loaders, buttons, app-state motion, product graphics, characters, storyboards, and scenes. If the user asks for a logo, loader, UI control, object, mascot, or anything else, create that subject's distinct editable parts. Prefer calm low-energy motion: subtle breathing, tiny bobs, soft pauses, restrained acknowledgment, and readable state changes that do not demand attention. Low-energy does not imply mascot anatomy, a face, a pet, or a fixed body model. Avoid frantic motion, giant jumps, confetti storms, speed lines, heavy squash/stretch, camera moves, and noisy effects unless explicitly requested. Use 8 to 16 editable nodes unless the user asks for a complex scene. Return compact JSON; do not pretty-print. Do not return a preset selector such as variant/name/accent/shell."##;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalGenerationKind {
    OllamaHttp,
    StdinPrompt,
    AcpOnly,
}

#[derive(Debug, Clone)]
struct LocalAdapterDefinition {
    id: &'static str,
    name: &'static str,
    kind: &'static str,
    commands: &'static [&'static str],
    version_args: &'static [&'static str],
    generation: LocalGenerationKind,
}

#[derive(Debug)]
struct CommandRun {
    ok: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
struct AgentAdapterStatus {
    id: String,
    name: String,
    kind: String,
    command: Option<String>,
    installed: bool,
    detail: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ByokProviderConfig {
    provider_id: String,
    api_key: Option<String>,
    endpoint: String,
    model: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerationProvider {
    mode: String,
    local_adapter_id: Option<String>,
    byok: Option<ByokProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceImageInput {
    name: String,
    mime_type: String,
    data_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerationContext {
    project_name: Option<String>,
    project_path: Option<String>,
    active_chat_title: Option<String>,
    current_document_summary: Option<String>,
    chat_history: Vec<GenerationContextMessage>,
    current_document: Option<strut_core::Document>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerationContextMessage {
    role: String,
    text: String,
    attachments: Option<Vec<String>>,
}

#[derive(Debug)]
struct WrittenReferenceFiles {
    directory: PathBuf,
    paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderOperationResult {
    ok: bool,
    status: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
struct GeneratedCharacter {
    document: strut_core::Document,
    source: String,
    message: String,
    plan_summary: Option<GenerationPlanSummary>,
    operation_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationPlanSummary {
    subject_classification: String,
    subject_label: String,
    part_names: Vec<String>,
    timeline_names: Vec<String>,
}

#[derive(Debug, Clone)]
struct PlannedDocument {
    document: strut_core::Document,
    summary: GenerationPlanSummary,
    operation_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerationPlanEnvelope {
    plan: GenerationPlan,
    #[serde(default)]
    operations: Vec<SceneOperation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerationPlan {
    id: Option<String>,
    name: String,
    subject: SubjectPlan,
    #[serde(default)]
    parts: Vec<SemanticPartPlan>,
    #[serde(default)]
    motion_roles: Vec<MotionRolePlan>,
    #[serde(default)]
    states: Vec<String>,
    #[serde(default)]
    timelines: Vec<TimelinePlan>,
    #[serde(default)]
    editability: EditabilityPlan,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubjectPlan {
    classification: String,
    label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemanticPartPlan {
    id: String,
    name: String,
    role: String,
    geometry: PlanGeometry,
    #[serde(default)]
    style: PlanStyle,
    #[serde(default)]
    motion_roles: Vec<String>,
    #[serde(default)]
    constraints: EditabilityConstraint,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanGeometry {
    kind: String,
    x: Option<f64>,
    y: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
    rx: Option<f64>,
    ry: Option<f64>,
    cx: Option<f64>,
    cy: Option<f64>,
    d: Option<String>,
    value: Option<String>,
    size: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanStyle {
    fill: Option<String>,
    stroke: Option<String>,
    #[serde(alias = "stroke_width")]
    stroke_width: Option<f64>,
    opacity: Option<f64>,
}

impl Default for PlanStyle {
    fn default() -> Self {
        Self {
            fill: Some("#f6f0df".to_string()),
            stroke: Some("#25221d".to_string()),
            stroke_width: Some(5.0),
            opacity: Some(1.0),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditabilityConstraint {
    #[serde(default = "default_editable")]
    editable: bool,
    #[serde(default)]
    allowed_properties: Vec<String>,
}

impl Default for EditabilityConstraint {
    fn default() -> Self {
        Self {
            editable: true,
            allowed_properties: Vec::new(),
        }
    }
}

fn default_editable() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditabilityPlan {
    #[serde(default)]
    editable_parts: Vec<String>,
    #[serde(default)]
    locked_parts: Vec<String>,
    #[serde(default)]
    notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MotionRolePlan {
    id: String,
    purpose: String,
    #[serde(default)]
    part_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimelinePlan {
    id: String,
    name: String,
    state: Option<String>,
    duration_ms: u32,
    #[serde(default)]
    tracks: Vec<TimelineTrackPlan>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimelineTrackPlan {
    target: String,
    property: String,
    #[serde(default)]
    keyframes: Vec<KeyframePlan>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeyframePlan {
    time_ms: u32,
    value: f64,
    easing: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SceneOperation {
    CreateNode {
        id: String,
        name: String,
        kind: String,
        parent: Option<String>,
        geometry: PlanGeometry,
        #[serde(default)]
        style: PlanStyle,
        role: Option<String>,
    },
    GroupNodes {
        id: String,
        name: String,
        children: Vec<String>,
    },
    SetProperty {
        target: String,
        property: String,
        value: Value,
    },
    AddState {
        state: String,
    },
    AddTimeline {
        id: String,
        name: String,
        state: Option<String>,
        duration_ms: u32,
    },
    AddKeyframe {
        timeline: String,
        target: String,
        property: String,
        time_ms: u32,
        value: f64,
        easing: Option<String>,
    },
    BindProperty {
        name: String,
        target: String,
        property: String,
    },
    EmitEvent {
        name: String,
        description: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedByokProviderConfig {
    provider_id: String,
    endpoint: String,
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectInfo {
    name: String,
    path: String,
    files: Vec<ProjectFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFile {
    name: String,
    path: String,
    kind: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationValidationResult {
    ok: bool,
    message: String,
    validator: String,
    validated_at: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationBatchRecord {
    id: String,
    source_type: String,
    status: String,
    validation_result: OperationValidationResult,
    document_revision_id: String,
    previous_document_revision_id: Option<String>,
    prompt: Option<String>,
    source_metadata: Option<Value>,
    operations: Vec<Value>,
    created_at: String,
    updated_at: String,
    applied_at: Option<String>,
    rejected_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedSelectionState {
    active_state: String,
    selected_node_id: Option<String>,
    layer_ui: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSnapshot {
    project: ProjectInfo,
    document: strut_core::Document,
    operation_batches: Vec<OperationBatchRecord>,
    selection: Option<PersistedSelectionState>,
    main_scene: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidatedGeneratedBatch {
    document: strut_core::Document,
    batch: OperationBatchRecord,
    plan_summary: GenerationPlanSummary,
    operation_count: usize,
}

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
fn default_project_location() -> String {
    default_projects_dir().display().to_string()
}

const PROJECT_MANIFEST_FILE: &str = "strut.project.json";
const MAIN_SCENE_FILE: &str = "scenes/main.strut";
const LEGACY_STARTER_SCENE_FILE: &str = "scenes/starter.strut.json";
const OPERATION_BATCHES_FILE: &str = "operations/operation-batches.json";
const STUDIO_STATE_FILE: &str = "ui/studio-state.json";

#[tauri::command]
fn create_project(name: String, location: String) -> Result<ProjectInfo, String> {
    let project_name = sanitize_project_name(&name)?;
    let root = if location.trim().is_empty() {
        default_projects_dir()
    } else {
        PathBuf::from(location.trim())
    };
    let project_path = root.join(&project_name);

    fs::create_dir_all(project_path.join("scenes")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join("assets")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join("exports")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join("operations")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join("ui")).map_err(|error| error.to_string())?;

    let document = strut_core::Document::empty_scene(&project_name);
    let scene_path = project_path.join(MAIN_SCENE_FILE);
    strut_format::write_strut_file(&scene_path, &strut_format::StrutPackage::current(document))
        .map_err(|error| error.to_string())?;

    fs::write(project_path.join(OPERATION_BATCHES_FILE), "[]")
        .map_err(|error| error.to_string())?;
    fs::write(
        project_path.join(STUDIO_STATE_FILE),
        serde_json::to_string_pretty(&PersistedSelectionState {
            active_state: "idle".to_string(),
            selected_node_id: None,
            layer_ui: json!({}),
        })
        .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let metadata = project_manifest_value(&project_name, unix_timestamp());
    fs::write(
        project_path.join(PROJECT_MANIFEST_FILE),
        serde_json::to_string_pretty(&metadata).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(project_info(project_name, project_path))
}

#[tauri::command]
fn save_project_snapshot(
    project_path: String,
    project_name: String,
    document: strut_core::Document,
    operation_batches: Vec<OperationBatchRecord>,
    selection: Option<PersistedSelectionState>,
) -> Result<ProjectSnapshot, String> {
    let root = ensure_project_root(&project_path)?;
    let project_name = sanitize_project_name(&project_name)?;
    strut_format::validate_document(&document).map_err(|error| error.to_string())?;
    validate_operation_batches(&operation_batches, &document)?;

    fs::create_dir_all(root.join("scenes")).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join("operations")).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join("ui")).map_err(|error| error.to_string())?;

    let scene_path = root.join(MAIN_SCENE_FILE);
    strut_format::write_strut_file(
        &scene_path,
        &strut_format::StrutPackage::current(document.clone()),
    )
    .map_err(|error| error.to_string())?;

    fs::write(
        root.join(OPERATION_BATCHES_FILE),
        serde_json::to_string_pretty(&operation_batches).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    if let Some(selection_state) = &selection {
        fs::write(
            root.join(STUDIO_STATE_FILE),
            serde_json::to_string_pretty(selection_state).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    }

    fs::write(
        root.join(PROJECT_MANIFEST_FILE),
        serde_json::to_string_pretty(&project_manifest_value(&project_name, unix_timestamp()))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(ProjectSnapshot {
        project: project_info(project_name, root),
        document,
        operation_batches,
        selection,
        main_scene: MAIN_SCENE_FILE.to_string(),
    })
}

#[tauri::command]
fn load_project_snapshot(project_path: String) -> Result<ProjectSnapshot, String> {
    let root = ensure_project_root(&project_path)?;
    let manifest_path = root.join(PROJECT_MANIFEST_FILE);
    let manifest = if manifest_path.exists() {
        let raw = fs::read_to_string(&manifest_path).map_err(|error| error.to_string())?;
        serde_json::from_str::<Value>(&raw).map_err(|error| error.to_string())?
    } else {
        json!({})
    };
    let project_name = manifest
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            root.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "Strut Project".to_string());
    let main_scene = manifest
        .get("mainScene")
        .and_then(Value::as_str)
        .unwrap_or(MAIN_SCENE_FILE)
        .to_string();
    let document = read_project_document(&root, &main_scene)?;
    let operation_batches = read_operation_batches(&root)?;
    validate_operation_batches(&operation_batches, &document)?;
    let selection = read_selection_state(&root)?;

    Ok(ProjectSnapshot {
        project: project_info(project_name, root),
        document,
        operation_batches,
        selection,
        main_scene,
    })
}

#[tauri::command]
fn validate_scene_document(document: strut_core::Document) -> OperationValidationResult {
    validation_result(strut_format::validate_document(&document).map_err(|error| error.to_string()))
}

#[tauri::command]
fn validate_generation_plan_batch(
    source_text: String,
    source_type: String,
    prompt: Option<String>,
) -> Result<ValidatedGeneratedBatch, String> {
    let planned = document_from_generation_plan_text(&source_text)?;
    let operations = operation_values_from_generation_plan_text(&source_text);
    let timestamp = timestamp_label();
    let revision = document_revision_id(&planned.document);
    let batch = OperationBatchRecord {
        id: format!(
            "batch-{}-{}-{}",
            sanitize_token(&source_type),
            revision,
            planned.operation_count
        ),
        source_type,
        status: "applied".to_string(),
        validation_result: OperationValidationResult {
            ok: true,
            message: "Rust validated generation plan operations before persistence".to_string(),
            validator: "strut-studio-rust".to_string(),
            validated_at: timestamp.clone(),
        },
        document_revision_id: revision,
        previous_document_revision_id: None,
        prompt,
        source_metadata: Some(json!({
            "subjectClassification": planned.summary.subject_classification,
            "subjectLabel": planned.summary.subject_label,
            "operationCount": planned.operation_count
        })),
        operations,
        created_at: timestamp.clone(),
        updated_at: timestamp.clone(),
        applied_at: Some(timestamp),
        rejected_at: None,
    };

    Ok(ValidatedGeneratedBatch {
        document: planned.document,
        batch,
        plan_summary: planned.summary,
        operation_count: planned.operation_count,
    })
}

#[tauri::command]
fn open_project_folder(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("No project folder is selected".to_string());
    }

    let folder = PathBuf::from(trimmed);
    if !folder.exists() {
        return Err(format!(
            "Project folder does not exist: {}",
            folder.display()
        ));
    }
    if !folder.is_dir() {
        return Err(format!(
            "Project path is not a folder: {}",
            folder.display()
        ));
    }

    open_folder_in_file_manager(&folder)
}

#[tauri::command]
async fn generate_character(
    prompt: String,
    provider: Option<GenerationProvider>,
    references: Option<Vec<ReferenceImageInput>>,
    context: Option<GenerationContext>,
) -> Result<GeneratedCharacter, String> {
    let references = references.unwrap_or_default();
    let contextual_prompt = contextual_generation_prompt(&prompt, context.as_ref())?;
    let provider = provider.ok_or_else(|| {
        "Select a real local CLI, Ollama, or BYOK provider before generating. Built-in fallback generation has been removed.".to_string()
    })?;

    match provider.mode.as_str() {
        "byok" => {
            let config = provider
                .byok
                .ok_or_else(|| "BYOK provider config missing".to_string())?;
            let document =
                generate_document_with_byok(&contextual_prompt, &config, &references).await?;
            Ok(GeneratedCharacter {
                source: config.provider_id,
                message: reference_message("Generated through BYOK provider", &references),
                plan_summary: Some(document.summary),
                operation_count: Some(document.operation_count),
                document: document.document,
            })
        }
        "local" => {
            let adapter_id = provider
                .local_adapter_id
                .ok_or_else(|| "Select a local CLI or Ollama adapter".to_string())?;
            let document =
                generate_document_with_local_adapter(&adapter_id, &contextual_prompt, &references)
                    .await?;
            Ok(GeneratedCharacter {
                source: adapter_id,
                message: reference_message("Generated through local provider", &references),
                plan_summary: Some(document.summary),
                operation_count: Some(document.operation_count),
                document: document.document,
            })
        }
        _ => Err("Unknown generation provider mode".to_string()),
    }
}

#[tauri::command]
fn local_agent_adapters() -> Vec<AgentAdapterStatus> {
    local_adapter_definitions()
        .into_iter()
        .map(|definition| {
            let resolved = resolve_adapter_command(&definition);
            let installed = resolved.is_some();
            let detail = match (&resolved, definition.generation) {
                (Some(path), LocalGenerationKind::AcpOnly) => format!(
                    "{} detected at {}; ACP generation is not implemented yet",
                    definition.commands[0],
                    path.display()
                ),
                (Some(path), _) => {
                    format!("{} found at {}", definition.commands[0], path.display())
                }
                (None, LocalGenerationKind::OllamaHttp) => {
                    "ollama not found on PATH; install Ollama and pull a model before testing"
                        .to_string()
                }
                (None, _) => format!(
                    "{} not found on PATH or common tool directories",
                    definition.commands.join(" / ")
                ),
            };

            AgentAdapterStatus {
                id: definition.id.to_string(),
                name: definition.name.to_string(),
                kind: definition.kind.to_string(),
                command: resolved.map(|path| path.display().to_string()),
                installed,
                detail,
            }
        })
        .collect()
}

#[tauri::command]
fn test_local_adapter(adapter_id: String) -> ProviderOperationResult {
    let Some(definition) = local_adapter_definitions()
        .into_iter()
        .find(|definition| definition.id == adapter_id)
    else {
        return ProviderOperationResult {
            ok: false,
            status: "unknown adapter".to_string(),
            detail: format!("{adapter_id} is not registered"),
        };
    };

    let Some(command) = resolve_adapter_command(&definition) else {
        return ProviderOperationResult {
            ok: false,
            status: format!("{} command missing", definition.name),
            detail: format!(
                "{} was not found on PATH or common tool directories",
                definition.commands.join(" / ")
            ),
        };
    };

    if definition.generation == LocalGenerationKind::AcpOnly {
        return ProviderOperationResult {
            ok: false,
            status: format!("{} detected, ACP pending", definition.name),
            detail: "This runtime exposes an Agent Client Protocol server. Strut detects it now, but generation is disabled until the ACP transport is implemented.".to_string(),
        };
    }

    let result = if definition.generation == LocalGenerationKind::OllamaHttp {
        run_ollama_smoke_test(&command)
    } else {
        let prompt = local_character_prompt(
            "Create a small floating helper named Smoke Bot with a cyan accent.",
            &[],
            None,
        );
        run_local_cli_command(
            &definition,
            &command,
            None,
            &prompt,
            Duration::from_secs(60),
        )
    };

    match result {
        Ok(output) if output.ok => ProviderOperationResult {
            ok: true,
            status: format!("{} ready", definition.name),
            detail: command_output_preview(&output.stdout, &output.stderr),
        },
        Ok(output) => ProviderOperationResult {
            ok: false,
            status: format!("{} returned an error", definition.name),
            detail: command_output_preview(&output.stdout, &output.stderr),
        },
        Err(error) => ProviderOperationResult {
            ok: false,
            status: format!("{} failed real smoke test", definition.name),
            detail: error,
        },
    }
}

#[tauri::command]
async fn test_byok_provider(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
    ensure_byok_config(&config)?;
    let smoke_prompt =
        "Create a small floating helper animation named Smoke Bot with a cyan accent.";
    match generate_document_with_byok(smoke_prompt, &config, &[]).await {
        Ok(_) => Ok(ProviderOperationResult {
            ok: true,
            status: format!("{} ready", provider_label(&config.provider_id)),
            detail: "provider completed a real Strut document generation smoke test".to_string(),
        }),
        Err(error) => Ok(ProviderOperationResult {
            ok: false,
            status: format!("{} failed smoke test", provider_label(&config.provider_id)),
            detail: error,
        }),
    }
}

#[tauri::command]
fn save_byok_provider(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
    if config.provider_id.trim().is_empty()
        || config.endpoint.trim().is_empty()
        || config.model.trim().is_empty()
    {
        return Err("provider, endpoint, and model are required".to_string());
    }

    let saved = SavedByokProviderConfig {
        provider_id: config.provider_id,
        endpoint: config.endpoint,
        model: config.model,
    };
    let path = provider_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let content = serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?;
    fs::write(&path, content).map_err(|error| error.to_string())?;

    Ok(ProviderOperationResult {
        ok: true,
        status: "provider config saved".to_string(),
        detail: format!(
            "saved endpoint and model to {}; API keys stay in session memory",
            path.display()
        ),
    })
}

fn local_adapter_definitions() -> Vec<LocalAdapterDefinition> {
    vec![
        LocalAdapterDefinition {
            id: "ollama",
            name: "Ollama",
            kind: "local-model",
            commands: &["ollama"],
            version_args: &["--version"],
            generation: LocalGenerationKind::OllamaHttp,
        },
        LocalAdapterDefinition {
            id: "codex",
            name: "Codex",
            kind: "local-agent",
            commands: &["codex"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "gemini-cli",
            name: "Gemini CLI",
            kind: "local-agent",
            commands: &["gemini"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "claude-code",
            name: "Claude Code",
            kind: "local-agent",
            commands: &["claude", "openclaude"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "opencode",
            name: "OpenCode",
            kind: "local-agent",
            commands: &["opencode-cli", "opencode"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "cursor-agent",
            name: "Cursor Agent",
            kind: "local-agent",
            commands: &["cursor-agent"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "qwen",
            name: "Qwen Code",
            kind: "local-agent",
            commands: &["qwen"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "qoder",
            name: "Qoder CLI",
            kind: "local-agent",
            commands: &["qodercli", "qoder"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "copilot-cli",
            name: "Copilot CLI",
            kind: "local-agent",
            commands: &["copilot"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "antigravity",
            name: "Antigravity",
            kind: "local-agent",
            commands: &["antigravity"],
            version_args: &["--version"],
            generation: LocalGenerationKind::AcpOnly,
        },
        LocalAdapterDefinition {
            id: "kiro",
            name: "Kiro",
            kind: "local-agent",
            commands: &["kiro-cli", "kiro"],
            version_args: &["--version"],
            generation: LocalGenerationKind::AcpOnly,
        },
        LocalAdapterDefinition {
            id: "lm-studio",
            name: "LM Studio",
            kind: "local-model",
            commands: &["lms"],
            version_args: &["--version"],
            generation: LocalGenerationKind::AcpOnly,
        },
    ]
}

fn default_projects_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        return PathBuf::from(home).join("Documents").join("Strut Projects");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("Strut Projects")
}

fn sanitize_project_name(name: &str) -> Result<String, String> {
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

fn project_info(name: String, path: PathBuf) -> ProjectInfo {
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

fn project_manifest_value(name: &str, timestamp: u64) -> Value {
    json!({
        "name": name,
        "createdAt": timestamp,
        "updatedAt": timestamp,
        "format": "0.2.0",
        "mainScene": MAIN_SCENE_FILE,
        "operationBatches": OPERATION_BATCHES_FILE,
        "studioState": STUDIO_STATE_FILE
    })
}

fn ensure_project_root(path: &str) -> Result<PathBuf, String> {
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

fn safe_project_file_path(
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

fn read_project_document(root: &Path, main_scene: &str) -> Result<strut_core::Document, String> {
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

fn read_legacy_document_json(path: &Path) -> Result<strut_core::Document, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let document =
        serde_json::from_str::<strut_core::Document>(&raw).map_err(|error| error.to_string())?;
    strut_format::validate_document(&document).map_err(|error| error.to_string())?;
    Ok(document)
}

fn read_operation_batches(root: &Path) -> Result<Vec<OperationBatchRecord>, String> {
    let path = root.join(OPERATION_BATCHES_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str::<Vec<OperationBatchRecord>>(&raw).map_err(|error| error.to_string())
}

fn read_selection_state(root: &Path) -> Result<Option<PersistedSelectionState>, String> {
    let path = root.join(STUDIO_STATE_FILE);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str::<PersistedSelectionState>(&raw)
        .map(Some)
        .map_err(|error| error.to_string())
}

struct OperationValidationContext {
    node_ids: HashSet<String>,
    node_refs: HashSet<String>,
    timeline_refs: HashSet<String>,
    states: HashSet<String>,
    events: HashSet<String>,
}

impl OperationValidationContext {
    fn from_document(document: &strut_core::Document) -> Self {
        let mut node_ids = HashSet::new();
        let mut node_refs = HashSet::new();
        for artboard in &document.artboards {
            collect_operation_node_refs(&artboard.nodes, &mut node_ids, &mut node_refs);
        }

        let mut timeline_refs = HashSet::new();
        for timeline in &document.timelines {
            timeline_refs.insert(timeline.id.to_string());
            timeline_refs.insert(timeline.name.clone());
        }

        let states = document
            .state_machines
            .iter()
            .flat_map(|machine| machine.states.iter().cloned())
            .collect();
        let events = document
            .events
            .iter()
            .map(|event| event.name.clone())
            .collect();

        Self {
            node_ids,
            node_refs,
            timeline_refs,
            states,
            events,
        }
    }

    fn has_node_id(&self, value: &str) -> bool {
        self.node_ids.contains(value)
    }

    fn has_node_ref(&self, value: &str) -> bool {
        self.node_refs.contains(value)
    }
}

struct GeneratedOperationRefs {
    node_refs: HashSet<String>,
    timeline_refs: HashSet<String>,
}

impl GeneratedOperationRefs {
    fn from_operations(operations: &[Value]) -> Self {
        let mut node_refs = HashSet::new();
        let mut timeline_refs = HashSet::new();
        for operation in operations {
            match operation.get("type").and_then(Value::as_str) {
                Some("create_node") => {
                    if let Some(id) = operation
                        .get("id")
                        .and_then(Value::as_str)
                        .filter(|id| !id.trim().is_empty())
                    {
                        node_refs.insert(id.to_string());
                    }
                }
                Some("add_timeline") => {
                    for field in ["id", "name"] {
                        if let Some(value) = operation
                            .get(field)
                            .and_then(Value::as_str)
                            .filter(|value| !value.trim().is_empty())
                        {
                            timeline_refs.insert(value.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        Self {
            node_refs,
            timeline_refs,
        }
    }

    fn has_node_ref(&self, value: &str) -> bool {
        self.node_refs.contains(value)
    }

    fn has_timeline_ref(&self, value: &str) -> bool {
        self.timeline_refs.contains(value)
    }
}

fn collect_operation_node_refs(
    nodes: &[strut_core::Node],
    node_ids: &mut HashSet<String>,
    node_refs: &mut HashSet<String>,
) {
    for node in nodes {
        let id = node.id.to_string();
        node_ids.insert(id.clone());
        node_refs.insert(id);
        node_refs.insert(node.name.clone());
        collect_operation_node_refs(&node.children, node_ids, node_refs);
    }
}

fn validate_operation_batches(
    batches: &[OperationBatchRecord],
    document: &strut_core::Document,
) -> Result<(), String> {
    let context = OperationValidationContext::from_document(document);
    let mut ids = HashSet::new();
    for batch in batches {
        if batch.id.trim().is_empty() {
            return Err("operation batch id is required".to_string());
        }
        if !ids.insert(batch.id.as_str()) {
            return Err(format!("duplicate operation batch id '{}'", batch.id));
        }
        if !matches!(
            batch.source_type.as_str(),
            "ai" | "sprite-python" | "manual" | "cli"
        ) {
            return Err(format!(
                "operation batch '{}' has unsupported source type '{}'",
                batch.id, batch.source_type
            ));
        }
        if !matches!(
            batch.status.as_str(),
            "pending" | "applied" | "rejected" | "undone"
        ) {
            return Err(format!(
                "operation batch '{}' has unsupported status '{}'",
                batch.id, batch.status
            ));
        }
        if batch.document_revision_id.trim().is_empty() {
            return Err(format!(
                "operation batch '{}' needs a document revision id",
                batch.id
            ));
        }
        if batch.created_at.trim().is_empty() || batch.updated_at.trim().is_empty() {
            return Err(format!("operation batch '{}' needs timestamps", batch.id));
        }
        if batch.status == "applied" && !batch.validation_result.ok {
            return Err(format!(
                "operation batch '{}' cannot be applied with failed validation",
                batch.id
            ));
        }
        if matches!(batch.status.as_str(), "pending" | "applied" | "undone")
            && batch.operations.is_empty()
        {
            return Err(format!(
                "operation batch '{}' has no meaningful operations",
                batch.id
            ));
        }
        validate_operation_batch_revision(batch)?;
        validate_operation_payloads(batch, &context)?;
    }
    Ok(())
}

fn validate_operation_batch_revision(batch: &OperationBatchRecord) -> Result<(), String> {
    if !batch.document_revision_id.starts_with("rev-") {
        return Err(format!(
            "operation batch '{}' has unsupported document revision id '{}'",
            batch.id, batch.document_revision_id
        ));
    }
    if let Some(previous) = &batch.previous_document_revision_id {
        if previous.trim().is_empty() {
            return Err(format!(
                "operation batch '{}' has an empty previous document revision id",
                batch.id
            ));
        }
    }
    Ok(())
}

fn validate_operation_payloads(
    batch: &OperationBatchRecord,
    context: &OperationValidationContext,
) -> Result<(), String> {
    let generated_refs = GeneratedOperationRefs::from_operations(&batch.operations);

    for operation in &batch.operations {
        validate_operation_payload(batch, operation, context, &generated_refs)?;
    }
    Ok(())
}

fn validate_operation_payload(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
    generated_refs: &GeneratedOperationRefs,
) -> Result<(), String> {
    let operation_type = operation
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            format!(
                "operation batch '{}' contains a malformed operation",
                batch.id
            )
        })?;

    match operation_type {
        "set_property" => validate_set_property_operation(batch, operation, context),
        "replace_document" => validate_replace_document_operation(batch, operation),
        "create_node" => validate_create_node_operation(batch, operation),
        "group_nodes" => validate_group_nodes_operation(batch, operation, context, generated_refs),
        "add_state" => validate_add_state_operation(batch, operation, context),
        "add_timeline" => validate_add_timeline_operation(batch, operation, context),
        "add_keyframe" => {
            validate_add_keyframe_operation(batch, operation, context, generated_refs)
        }
        "bind_property" => {
            validate_bind_property_operation(batch, operation, context, generated_refs)
        }
        "emit_event" => validate_emit_event_operation(batch, operation, context),
        other => Err(format!(
            "operation batch '{}' contains unsupported operation type '{}'",
            batch.id, other
        )),
    }
}

fn validate_set_property_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
) -> Result<(), String> {
    let target_id = required_string_field(batch, operation, "targetId")?;
    if !context.has_node_id(target_id) {
        return Err(format!(
            "operation batch '{}' targets unknown node id '{}'",
            batch.id, target_id
        ));
    }

    let property = required_string_field(batch, operation, "property")?;
    let value = operation.get("value").ok_or_else(|| {
        format!(
            "operation batch '{}' set_property operation needs a value",
            batch.id
        )
    })?;
    validate_set_property_value(batch, property, value)?;
    if let Some(previous_value) = operation.get("previousValue") {
        validate_set_property_value(batch, property, previous_value)?;
    }
    Ok(())
}

fn validate_set_property_value(
    batch: &OperationBatchRecord,
    property: &str,
    value: &Value,
) -> Result<(), String> {
    match property {
        "style.fill" | "style.stroke" => {
            if value.is_null() || value.as_str().is_some() {
                Ok(())
            } else {
                Err(format!(
                    "operation batch '{}' has invalid value for property '{}'",
                    batch.id, property
                ))
            }
        }
        "style.opacity" => validate_finite_number_range(batch, property, value, 0.0, 1.0),
        "style.stroke_width" => validate_finite_number_range(batch, property, value, 0.0, f64::MAX),
        "transform.translate_x" | "transform.translate_y" | "transform.rotate" => {
            validate_finite_number(batch, property, value)
        }
        "transform.scale_x" | "transform.scale_y" => {
            validate_finite_number_range(batch, property, value, f64::MIN_POSITIVE, f64::MAX)
        }
        _ => Err(format!(
            "operation batch '{}' uses unsupported set_property path '{}'",
            batch.id, property
        )),
    }
}

fn validate_replace_document_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
) -> Result<(), String> {
    let next_document = operation.get("nextDocument").ok_or_else(|| {
        format!(
            "operation batch '{}' replace_document operation needs nextDocument",
            batch.id
        )
    })?;
    validate_document_value(batch, next_document, "nextDocument")?;

    if let Some(previous_document) = operation.get("previousDocument") {
        if !previous_document.is_null() {
            validate_document_value(batch, previous_document, "previousDocument")?;
        }
    }
    Ok(())
}

fn validate_document_value(
    batch: &OperationBatchRecord,
    value: &Value,
    field: &str,
) -> Result<(), String> {
    let document =
        serde_json::from_value::<strut_core::Document>(value.clone()).map_err(|error| {
            format!(
                "operation batch '{}' has invalid replacement document in {field}: {error}",
                batch.id
            )
        })?;
    strut_format::validate_document(&document).map_err(|error| {
        format!(
            "operation batch '{}' replacement document in {field} failed validation: {error}",
            batch.id
        )
    })
}

fn validate_create_node_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
) -> Result<(), String> {
    let id = required_string_field(batch, operation, "id")?;
    let name = required_string_field(batch, operation, "name")?;
    let kind = required_string_field(batch, operation, "kind")?;
    if id.trim().is_empty() || name.trim().is_empty() {
        return Err(format!(
            "operation batch '{}' create_node operation needs stable id and name",
            batch.id
        ));
    }
    if !matches!(
        kind,
        "group" | "rect" | "rectangle" | "ellipse" | "path" | "text"
    ) {
        return Err(format!(
            "operation batch '{}' create_node operation has unsupported kind '{}'",
            batch.id, kind
        ));
    }
    let geometry = operation.get("geometry").ok_or_else(|| {
        format!(
            "operation batch '{}' create_node operation needs geometry",
            batch.id
        )
    })?;
    let geometry = serde_json::from_value::<PlanGeometry>(geometry.clone()).map_err(|error| {
        format!(
            "operation batch '{}' create_node operation has malformed geometry: {error}",
            batch.id
        )
    })?;
    validate_plan_geometry(id, &geometry)
}

fn validate_group_nodes_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
    generated_refs: &GeneratedOperationRefs,
) -> Result<(), String> {
    required_string_field(batch, operation, "id")?;
    required_string_field(batch, operation, "name")?;
    let children = operation
        .get("children")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("operation batch '{}' group_nodes needs children", batch.id))?;
    if children.is_empty() {
        return Err(format!(
            "operation batch '{}' group_nodes has no children",
            batch.id
        ));
    }
    for child in children {
        let Some(child) = child.as_str() else {
            return Err(format!(
                "operation batch '{}' group_nodes contains a malformed child id",
                batch.id
            ));
        };
        if !context.has_node_ref(child) && !generated_refs.has_node_ref(child) {
            return Err(format!(
                "operation batch '{}' group_nodes references unknown child '{}'",
                batch.id, child
            ));
        }
    }
    Ok(())
}

fn validate_add_state_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
) -> Result<(), String> {
    let state = required_string_field(batch, operation, "state")?;
    if !context.states.contains(state) {
        return Err(format!(
            "operation batch '{}' add_state references unknown state '{}'",
            batch.id, state
        ));
    }
    Ok(())
}

fn validate_add_timeline_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
) -> Result<(), String> {
    required_string_field(batch, operation, "id")?;
    required_string_field(batch, operation, "name")?;
    let duration = operation
        .get("duration_ms")
        .and_then(Value::as_u64)
        .or_else(|| operation.get("durationMs").and_then(Value::as_u64))
        .ok_or_else(|| {
            format!(
                "operation batch '{}' add_timeline needs a positive duration",
                batch.id
            )
        })?;
    if duration == 0 {
        return Err(format!(
            "operation batch '{}' add_timeline needs a positive duration",
            batch.id
        ));
    }
    if let Some(state) = operation.get("state").and_then(Value::as_str) {
        if !context.states.contains(state) {
            return Err(format!(
                "operation batch '{}' add_timeline references unknown state '{}'",
                batch.id, state
            ));
        }
    }
    Ok(())
}

fn validate_add_keyframe_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
    generated_refs: &GeneratedOperationRefs,
) -> Result<(), String> {
    let timeline = required_string_field(batch, operation, "timeline")?;
    if !context.timeline_refs.contains(timeline) && !generated_refs.has_timeline_ref(timeline) {
        return Err(format!(
            "operation batch '{}' add_keyframe references unknown timeline '{}'",
            batch.id, timeline
        ));
    }
    let target = required_string_field(batch, operation, "target")?;
    if !context.has_node_ref(target) && !generated_refs.has_node_ref(target) {
        return Err(format!(
            "operation batch '{}' add_keyframe targets unknown node '{}'",
            batch.id, target
        ));
    }
    let property = required_string_field(batch, operation, "property")?;
    if !allowed_timeline_property(property) {
        return Err(format!(
            "operation batch '{}' add_keyframe uses unsupported property '{}'",
            batch.id, property
        ));
    }
    if operation.get("time_ms").and_then(Value::as_u64).is_none()
        && operation.get("timeMs").and_then(Value::as_u64).is_none()
    {
        return Err(format!(
            "operation batch '{}' add_keyframe needs time_ms",
            batch.id
        ));
    }
    let value = operation.get("value").ok_or_else(|| {
        format!(
            "operation batch '{}' add_keyframe needs a numeric value",
            batch.id
        )
    })?;
    validate_finite_number(batch, property, value)
}

fn validate_bind_property_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
    generated_refs: &GeneratedOperationRefs,
) -> Result<(), String> {
    required_string_field(batch, operation, "name")?;
    let target = required_string_field(batch, operation, "target")?;
    if !context.has_node_ref(target) && !generated_refs.has_node_ref(target) {
        return Err(format!(
            "operation batch '{}' bind_property targets unknown node '{}'",
            batch.id, target
        ));
    }
    let property = required_string_field(batch, operation, "property")?;
    if !allowed_edit_property(property) {
        return Err(format!(
            "operation batch '{}' bind_property uses unsupported property '{}'",
            batch.id, property
        ));
    }
    Ok(())
}

fn validate_emit_event_operation(
    batch: &OperationBatchRecord,
    operation: &Value,
    context: &OperationValidationContext,
) -> Result<(), String> {
    let name = required_string_field(batch, operation, "name")?;
    if !context.events.contains(name) {
        return Err(format!(
            "operation batch '{}' emit_event references unknown event '{}'",
            batch.id, name
        ));
    }
    Ok(())
}

fn required_string_field<'a>(
    batch: &OperationBatchRecord,
    operation: &'a Value,
    field: &str,
) -> Result<&'a str, String> {
    operation
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "operation batch '{}' operation needs string field '{}'",
                batch.id, field
            )
        })
}

fn validate_finite_number(
    batch: &OperationBatchRecord,
    property: &str,
    value: &Value,
) -> Result<(), String> {
    let Some(number) = value.as_f64() else {
        return Err(format!(
            "operation batch '{}' has non-numeric value for property '{}'",
            batch.id, property
        ));
    };
    if number.is_finite() {
        Ok(())
    } else {
        Err(format!(
            "operation batch '{}' has non-finite value for property '{}'",
            batch.id, property
        ))
    }
}

fn validate_finite_number_range(
    batch: &OperationBatchRecord,
    property: &str,
    value: &Value,
    minimum: f64,
    maximum: f64,
) -> Result<(), String> {
    validate_finite_number(batch, property, value)?;
    let number = value.as_f64().unwrap_or_default();
    if number >= minimum && number <= maximum {
        Ok(())
    } else {
        Err(format!(
            "operation batch '{}' has out-of-range value for property '{}'",
            batch.id, property
        ))
    }
}

fn validation_result(result: Result<(), String>) -> OperationValidationResult {
    let timestamp = timestamp_label();
    match result {
        Ok(()) => OperationValidationResult {
            ok: true,
            message: "document validated by Rust format rules".to_string(),
            validator: "strut-studio-rust".to_string(),
            validated_at: timestamp,
        },
        Err(error) => OperationValidationResult {
            ok: false,
            message: error,
            validator: "strut-studio-rust".to_string(),
            validated_at: timestamp,
        },
    }
}

fn operation_values_from_generation_plan_text(text: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(text.trim())
        .ok()
        .and_then(|value| value.get("operations").cloned())
        .and_then(|operations| operations.as_array().cloned())
        .unwrap_or_default()
}

fn document_revision_id(document: &strut_core::Document) -> String {
    format!(
        "rev-{}-{}-{}",
        sanitize_token(&document.name),
        document.artboards.len(),
        document.timelines.len()
    )
}

fn sanitize_token(value: &str) -> String {
    let token = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '-')
        .collect::<String>()
        .to_lowercase();
    if token.is_empty() {
        "unknown".to_string()
    } else {
        token
    }
}

fn timestamp_label() -> String {
    unix_timestamp().to_string()
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn resolve_adapter_command(definition: &LocalAdapterDefinition) -> Option<PathBuf> {
    definition
        .commands
        .iter()
        .find_map(|command| resolve_command(command))
}

fn resolve_command(command: &str) -> Option<PathBuf> {
    let candidates = command_candidates(command);
    command_search_dirs().into_iter().find_map(|path| {
        candidates
            .iter()
            .map(|candidate| path.join(candidate))
            .find(|candidate| candidate.is_file())
    })
}

fn command_search_dirs() -> Vec<PathBuf> {
    let mut dirs = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .unwrap_or_default();

    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let home = PathBuf::from(home);
        dirs.extend([
            home.join(".codex").join("bin"),
            home.join(".local").join("bin"),
            home.join(".npm-global").join("bin"),
            home.join("AppData").join("Roaming").join("npm"),
            home.join("AppData")
                .join("Local")
                .join("Programs")
                .join("cursor")
                .join("resources")
                .join("app")
                .join("bin"),
        ]);
    }

    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let program_files = PathBuf::from(program_files);
        dirs.extend([
            program_files.join("GitHub CLI"),
            program_files.join("Ollama"),
            program_files.join("Claude"),
        ]);
    }

    dirs
}

fn command_candidates(command: &str) -> Vec<PathBuf> {
    if cfg!(windows) {
        if Path::new(command).extension().is_some() {
            return vec![PathBuf::from(command)];
        }
        let extensions = std::env::var_os("PATHEXT")
            .map(|value| {
                value
                    .to_string_lossy()
                    .split(';')
                    .filter(|extension| !extension.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .filter(|extensions| !extensions.is_empty())
            .unwrap_or_else(|| vec![".EXE".to_string(), ".CMD".to_string(), ".BAT".to_string()]);

        let mut candidates = extensions
            .into_iter()
            .flat_map(|extension| {
                [
                    PathBuf::from(format!("{command}{extension}")),
                    PathBuf::from(format!("{command}{}", extension.to_lowercase())),
                ]
            })
            .collect::<Vec<_>>();
        candidates.push(PathBuf::from(command));
        candidates
    } else {
        vec![PathBuf::from(command)]
    }
}

fn command_output_preview(stdout: &[u8], stderr: &[u8]) -> String {
    let mut text = String::from_utf8_lossy(stdout).trim().to_string();
    if text.is_empty() {
        text = String::from_utf8_lossy(stderr).trim().to_string();
    }
    if text.is_empty() {
        "command completed without output".to_string()
    } else {
        text.chars().take(220).collect()
    }
}

fn response_preview(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        "empty response".to_string()
    } else {
        compact.chars().take(700).collect()
    }
}

fn document_repair_prompt(
    original_prompt: &str,
    invalid_response: &str,
    parse_error: &str,
) -> String {
    format!(
        "{CHARACTER_DOCUMENT_SYSTEM_PROMPT}\n\nThe previous response could not be loaded by Strut.\nValidation error:\n{parse_error}\n\nOriginal user request:\n{original_prompt}\n\nPrevious invalid response:\n{}\n\nRepair task: return one valid compact JSON object only in this exact shape: {{\"document\": <StrutDocument>}}. Keep the user's requested subject and animation intent. Use 8 to 12 editable nodes, short readable ids, five compact timelines, and one state machine. Do not explain, do not use markdown, do not return a preset spec, and do not omit timelines or state_machines.",
        response_preview(invalid_response)
    )
}

fn generation_plan_repair_prompt(
    original_prompt: &str,
    invalid_response: &str,
    parse_error: &str,
) -> String {
    format!(
        "{GENERATION_PLAN_SYSTEM_PROMPT}\n\nThe previous response could not be converted by Strut.\nValidation error:\n{parse_error}\n\nOriginal user request:\n{original_prompt}\n\nPrevious invalid response:\n{}\n\nRepair task: return one valid compact JSON object only in this exact shape: {{\"plan\": <GenerationPlan>, \"operations\": []}}. Keep the requested subject, use subject-specific semantic parts, include named states/timelines, and leave operations empty if unsure so Strut can derive validated operations. Do not explain, do not use markdown, do not return mascot anatomy unless the subject is a mascot.",
        response_preview(invalid_response)
    )
}

fn compact_plan_prompt(original_prompt: &str, previous_error: &str) -> String {
    format!(
        "{GENERATION_PLAN_SYSTEM_PROMPT}\n\nConvert this motion design request into a compact Strut generation plan.\nOriginal request: {original_prompt}\nPrevious attempt failed: {previous_error}\n\nReturn JSON only in this exact shape: {{\"plan\": <GenerationPlan>, \"operations\": []}}.\nRules: include 6 to 10 visually distinct parts that match the requested subject. Use absolute artboard coordinates. Include states, timelines, tracks, and editable constraints. The motion must be calm and low-energy: subtle bob, tiny tilt, focused scan, restrained settle, soft reveal, progress sweep, or similar. Do not explain."
    )
}

fn open_folder_in_file_manager(folder: &Path) -> Result<(), String> {
    let status = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(folder).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(folder).spawn()
    } else {
        Command::new("xdg-open").arg(folder).spawn()
    }
    .map_err(|error| format!("Could not open project folder: {error}"))?
    .wait()
    .map_err(|error| format!("Could not open project folder: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("File explorer exited with status {status}"))
    }
}

fn run_ollama_smoke_test(command: &Path) -> Result<CommandRun, String> {
    run_command_with_stdin(
        command,
        &["run".to_string(), "llama3.2".to_string()],
        &[],
        None,
        &local_character_prompt(
            "Create a small floating helper named Smoke Bot with a cyan accent.",
            &[],
            None,
        ),
        Duration::from_secs(60),
    )
}

fn run_local_cli_command(
    definition: &LocalAdapterDefinition,
    command: &Path,
    reference_dir: Option<&Path>,
    input: &str,
    timeout: Duration,
) -> Result<CommandRun, String> {
    let args = local_generation_args(definition, reference_dir);
    let env = local_generation_env(definition);
    run_command_with_stdin(command, &args, &env, None, input, timeout)
}

fn run_command_with_stdin(
    command: &Path,
    args: &[String],
    env: &[(&str, &str)],
    cwd: Option<&Path>,
    input: &str,
    timeout: Duration,
) -> Result<CommandRun, String> {
    let mut process = Command::new(command);
    process
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = cwd {
        process.current_dir(cwd);
    }
    for (key, value) in env {
        process.env(key, value);
    }

    let mut child = process.spawn().map_err(|error| error.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|error| error.to_string())?;
    }

    let started = Instant::now();
    loop {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            let output = child
                .wait_with_output()
                .map_err(|error| error.to_string())?;
            return Ok(CommandRun {
                ok: output.status.success(),
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }

        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "{} timed out after {} seconds",
                command.display(),
                timeout.as_secs()
            ));
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

fn local_generation_args(
    definition: &LocalAdapterDefinition,
    reference_dir: Option<&Path>,
) -> Vec<String> {
    match definition.id {
        "codex" => {
            let mut args = vec![
                "exec".to_string(),
                "--json".to_string(),
                "--skip-git-repo-check".to_string(),
            ];
            if cfg!(windows) {
                args.extend(["--sandbox".to_string(), "danger-full-access".to_string()]);
            } else {
                args.extend([
                    "--sandbox".to_string(),
                    "workspace-write".to_string(),
                    "-c".to_string(),
                    "sandbox_workspace_write.network_access=true".to_string(),
                ]);
            }
            if let Some(dir) = reference_dir {
                args.extend(["--add-dir".to_string(), dir.display().to_string()]);
            }
            args
        }
        "gemini-cli" => vec![
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--prompt".to_string(),
            "Generate exactly the requested JSON from stdin.".to_string(),
        ],
        "claude-code" => vec![
            "-p".to_string(),
            "--input-format".to_string(),
            "text".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
            "--permission-mode".to_string(),
            "bypassPermissions".to_string(),
        ],
        "opencode" => vec![
            "run".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--dangerously-skip-permissions".to_string(),
        ],
        "cursor-agent" => vec![
            "--print".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--stream-partial-output".to_string(),
            "--force".to_string(),
            "--trust".to_string(),
        ],
        "qwen" => vec!["--yolo".to_string()],
        "qoder" => vec![
            "-p".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--yolo".to_string(),
        ],
        "copilot-cli" => vec![
            "--allow-all-tools".to_string(),
            "--output-format".to_string(),
            "json".to_string(),
        ],
        _ => definition
            .version_args
            .iter()
            .map(|arg| arg.to_string())
            .collect(),
    }
}

fn local_generation_env(definition: &LocalAdapterDefinition) -> Vec<(&'static str, &'static str)> {
    match definition.id {
        "gemini-cli" => vec![("GEMINI_CLI_TRUST_WORKSPACE", "true")],
        _ => Vec::new(),
    }
}

fn provider_config_path() -> Result<PathBuf, String> {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return Ok(PathBuf::from(appdata)
            .join("Strut")
            .join("providers")
            .join("byok.json"));
    }

    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home)
            .join("strut")
            .join("providers")
            .join("byok.json"));
    }

    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home)
            .join(".config")
            .join("strut")
            .join("providers")
            .join("byok.json"));
    }

    Err("could not resolve a local config directory".to_string())
}

fn ensure_byok_config(config: &ByokProviderConfig) -> Result<(), String> {
    if config
        .api_key
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        return Err(format!(
            "{} API key is required",
            provider_label(&config.provider_id)
        ));
    }
    if config.endpoint.trim().is_empty() {
        return Err("provider endpoint is required".to_string());
    }
    ensure_safe_endpoint(&config.endpoint)?;
    if config.model.trim().is_empty() {
        return Err("provider model is required".to_string());
    }
    Ok(())
}

fn ensure_safe_endpoint(endpoint: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(endpoint.trim())
        .map_err(|error| format!("provider endpoint is not a valid URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("provider endpoint must use http or https".to_string());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "provider endpoint must include a host".to_string())?;
    if host.eq_ignore_ascii_case("localhost") {
        return Ok(());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if ip.is_loopback() {
            return Ok(());
        }
        if is_blocked_internal_ip(ip) {
            return Err("provider endpoint cannot target private, link-local, or unspecified network addresses unless it is loopback".to_string());
        }
    }
    Ok(())
}

fn is_blocked_internal_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private() || ip.is_link_local() || ip.is_unspecified() || ip.is_broadcast()
        }
        IpAddr::V6(ip) => ip.is_unique_local() || ip.is_unicast_link_local() || ip.is_unspecified(),
    }
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| error.to_string())
}

fn endpoint_base(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_string()
}

fn provider_label(provider_id: &str) -> &'static str {
    match provider_id {
        "openai" => "OpenAI",
        "anthropic" => "Anthropic",
        "gemini" => "Gemini",
        "openrouter" => "OpenRouter",
        "azure-openai" => "Azure OpenAI",
        "openai-compatible" => "OpenAI Compatible",
        _ => "Provider",
    }
}

fn http_error_preview(status: u16, body: &str) -> String {
    let preview: String = body.chars().take(260).collect();
    if preview.trim().is_empty() {
        format!("HTTP {status}")
    } else {
        format!("HTTP {status}: {preview}")
    }
}

fn data_url_payload(data_url: &str) -> Option<&str> {
    data_url.split_once(',').map(|(_, payload)| payload)
}

fn image_media_type(reference: &ReferenceImageInput) -> &str {
    if reference.mime_type.trim().is_empty() {
        return "image/png";
    }
    reference.mime_type.trim()
}

fn prompt_with_reference_context(prompt: &str, references: &[ReferenceImageInput]) -> String {
    if references.is_empty() {
        return prompt.to_string();
    }
    let names = references
        .iter()
        .map(|reference| {
            format!(
                "{} ({})",
                reference.name.trim(),
                image_media_type(reference)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{prompt}\n\nReference images attached: {names}. Inspect the image composition, silhouette, pose, palette, typography, geometry, and visible parts, then return an editable Strut motion document that matches the reference direction."
    )
}

fn contextual_generation_prompt(
    prompt: &str,
    context: Option<&GenerationContext>,
) -> Result<String, String> {
    let Some(context) = context else {
        return Ok(prompt.to_string());
    };

    let mut text = String::new();
    text.push_str("Strut workspace context:\n");
    if let Some(project_name) = context
        .project_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        text.push_str(&format!("- Project: {}\n", project_name.trim()));
    }
    if let Some(project_path) = context
        .project_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        text.push_str(&format!("- Project path: {}\n", project_path.trim()));
    }
    if let Some(chat_title) = context
        .active_chat_title
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        text.push_str(&format!("- Active chat: {}\n", chat_title.trim()));
    }
    if let Some(summary) = context
        .current_document_summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        text.push_str(&format!("- Current document: {}\n", summary.trim()));
    }

    if !context.chat_history.is_empty() {
        text.push_str("\nRecent chat history. Use it to resolve follow-up edits and pronouns:\n");
        for message in context.chat_history.iter().take(16) {
            let role = message.role.trim();
            let body = message.text.trim();
            if body.is_empty() && message.attachments.as_ref().map_or(true, Vec::is_empty) {
                continue;
            }
            text.push_str(&format!("- {}: {}", role, body));
            if let Some(attachments) = &message.attachments {
                let names = attachments
                    .iter()
                    .filter(|name| !name.trim().is_empty())
                    .map(|name| name.trim())
                    .collect::<Vec<_>>();
                if !names.is_empty() {
                    text.push_str(&format!(" [attachments: {}]", names.join(", ")));
                }
            }
            text.push('\n');
        }
    }

    if let Some(document) = &context.current_document {
        let document_json = serde_json::to_string_pretty(document)
            .map_err(|error| format!("Could not serialize current Strut document: {error}"))?;
        text.push_str(
            "\nCurrent editable Strut document. Treat the user request as an edit to this document unless they explicitly ask for a new scene. Preserve unaffected layers, states, timelines, bindings, and events. Return a subject-aware generation plan plus explicit operations; do not replace the whole document unless the fallback repair prompt specifically asks for it:\n",
        );
        text.push_str(&document_json);
        text.push('\n');
    }

    text.push_str("\nUser request:\n");
    text.push_str(prompt.trim());
    Ok(text)
}

fn local_character_prompt(
    prompt: &str,
    references: &[ReferenceImageInput],
    reference_files: Option<&WrittenReferenceFiles>,
) -> String {
    let mut text = format!(
        "{GENERATION_PLAN_SYSTEM_PROMPT}\n\nUser request:\n{}",
        prompt_with_reference_context(prompt, references)
    );
    if let Some(files) = reference_files {
        if !files.paths.is_empty() {
            text.push_str("\n\nReference files written for this run:");
            for path in &files.paths {
                text.push_str(&format!("\n- {}", path.display()));
            }
            text.push_str("\nUse those image files as visual references when your runtime supports local file inspection.");
        }
    }
    text.push_str(
        "\n\nDo not inspect files, run tools, edit the workspace, or explain your answer. Return only the JSON object. Prefer the generation plan and operations schema. Do not include markdown or a legacy variant spec.",
    );
    text
}

fn write_reference_files(
    references: &[ReferenceImageInput],
) -> Result<Option<WrittenReferenceFiles>, String> {
    if references.is_empty() {
        return Ok(None);
    }

    let directory = std::env::temp_dir().join(format!("strut-references-{}", unix_timestamp()));
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let mut paths = Vec::new();
    for (index, reference) in references.iter().enumerate() {
        let Some(payload) = data_url_payload(&reference.data_url) else {
            continue;
        };
        let bytes = general_purpose::STANDARD
            .decode(payload)
            .map_err(|error| format!("could not decode {}: {error}", reference.name))?;
        let extension = image_extension(reference);
        let name = sanitize_file_stem(&reference.name);
        let path = directory.join(format!("{index:02}-{name}.{extension}"));
        fs::write(&path, bytes).map_err(|error| error.to_string())?;
        paths.push(path);
    }

    Ok(Some(WrittenReferenceFiles { directory, paths }))
}

fn sanitize_file_stem(name: &str) -> String {
    let sanitized = name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .collect::<String>();
    if sanitized.is_empty() {
        "reference".to_string()
    } else {
        sanitized
    }
}

fn image_extension(reference: &ReferenceImageInput) -> &'static str {
    match image_media_type(reference) {
        "image/jpeg" => "jpg",
        "image/svg+xml" => "svg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "png",
    }
}

fn reference_message(base: &str, references: &[ReferenceImageInput]) -> String {
    if references.is_empty() {
        base.to_string()
    } else {
        format!(
            "{base} using {} reference image{}",
            references.len(),
            if references.len() == 1 { "" } else { "s" }
        )
    }
}

async fn generate_document_with_byok(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<PlannedDocument, String> {
    ensure_byok_config(config)?;
    let response_text = byok_generate_text(prompt, config, references).await?;

    match parse_provider_response_document(&response_text) {
        Ok(document) => Ok(document),
        Err(first_error) => {
            let repair_prompt = generation_plan_repair_prompt(prompt, &response_text, &first_error);
            let repaired_text = byok_generate_text(&repair_prompt, config, &[]).await?;
            match parse_provider_response_document(&repaired_text) {
                Ok(document) => Ok(document),
                Err(repair_error) => {
                    let plan_prompt = compact_plan_prompt(prompt, &repair_error);
                    let plan_text = byok_generate_text(&plan_prompt, config, &[]).await?;
                    document_from_generation_plan_text(&plan_text)
                        .or_else(|_| document_from_compact_plan_text(&plan_text).map(planned_from_compact_document))
                        .map_err(|plan_error| {
                        format!(
                            "model did not return a valid Strut document after repair. First error: {first_error}. Repair error: {repair_error}. Plan error: {plan_error}. Response preview: {}",
                            response_preview(&plan_text)
                        )
                    })
                }
            }
        }
    }
}

async fn byok_generate_text(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {
    Ok(match config.provider_id.as_str() {
        "anthropic" => anthropic_message(prompt, config, references).await?,
        "gemini" => gemini_generate_content(prompt, config, references).await?,
        _ => openai_compatible_chat(prompt, config, references).await?,
    })
}

async fn generate_document_with_local_adapter(
    adapter_id: &str,
    prompt: &str,
    references: &[ReferenceImageInput],
) -> Result<PlannedDocument, String> {
    let definition = local_adapter_definitions()
        .into_iter()
        .find(|definition| definition.id == adapter_id)
        .ok_or_else(|| format!("{adapter_id} is not registered"))?;

    if definition.generation == LocalGenerationKind::OllamaHttp {
        return generate_document_with_ollama(prompt, references).await;
    }

    if definition.generation == LocalGenerationKind::AcpOnly {
        return Err(format!(
            "{} uses an ACP-style runtime. Strut detects it, but real generation is disabled until ACP transport support lands.",
            definition.name
        ));
    }

    let command = resolve_adapter_command(&definition).ok_or_else(|| {
        format!(
            "{} was not found on PATH or common tool directories",
            definition.commands.join(" / ")
        )
    })?;
    let reference_files = write_reference_files(references)?;
    let reference_dir = reference_files
        .as_ref()
        .map(|files| files.directory.as_path());
    let local_prompt = local_character_prompt(prompt, references, reference_files.as_ref());
    let output = run_local_cli_command(
        &definition,
        &command,
        reference_dir,
        &local_prompt,
        Duration::from_secs(240),
    )?;
    let _ = reference_files
        .as_ref()
        .map(|files| fs::remove_dir_all(&files.directory));

    if !output.ok {
        return Err(command_output_preview(&output.stdout, &output.stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let assistant_text = cli_assistant_text(&stdout);
    match parse_provider_response_document(&assistant_text)
        .or_else(|_| parse_provider_response_document(&stdout))
        .or_else(|_| parse_provider_response_document(&stderr))
    {
        Ok(document) => Ok(document),
        Err(first_error) => {
            let invalid_response = if assistant_text.trim().is_empty() {
                format!("{stdout}\n{stderr}")
            } else {
                assistant_text
            };
            let repair_prompt = local_character_prompt(
                &generation_plan_repair_prompt(prompt, &invalid_response, &first_error),
                &[],
                None,
            );
            let repair_output = run_local_cli_command(
                &definition,
                &command,
                None,
                &repair_prompt,
                Duration::from_secs(300),
            )?;
            if !repair_output.ok {
                return Err(command_output_preview(
                    &repair_output.stdout,
                    &repair_output.stderr,
                ));
            }
            let repair_stdout = String::from_utf8_lossy(&repair_output.stdout).to_string();
            let repair_stderr = String::from_utf8_lossy(&repair_output.stderr).to_string();
            let repair_text = cli_assistant_text(&repair_stdout);
            match parse_provider_response_document(&repair_text)
                .or_else(|_| parse_provider_response_document(&repair_stdout))
                .or_else(|_| parse_provider_response_document(&repair_stderr))
            {
                Ok(document) => Ok(document),
                Err(repair_error) => {
                    let plan_prompt = local_character_prompt(
                        &compact_plan_prompt(prompt, &repair_error),
                        &[],
                        None,
                    );
                    let plan_output = run_local_cli_command(
                        &definition,
                        &command,
                        None,
                        &plan_prompt,
                        Duration::from_secs(180),
                    )?;
                    if !plan_output.ok {
                        return Err(command_output_preview(
                            &plan_output.stdout,
                            &plan_output.stderr,
                        ));
                    }
                    let plan_stdout = String::from_utf8_lossy(&plan_output.stdout).to_string();
                    let plan_stderr = String::from_utf8_lossy(&plan_output.stderr).to_string();
                    let plan_text = cli_assistant_text(&plan_stdout);
                    document_from_generation_plan_text(&plan_text)
                        .or_else(|_| document_from_generation_plan_text(&plan_stdout))
                        .or_else(|_| document_from_generation_plan_text(&plan_stderr))
                        .or_else(|_| document_from_compact_plan_text(&plan_text).map(planned_from_compact_document))
                        .or_else(|_| document_from_compact_plan_text(&plan_stdout).map(planned_from_compact_document))
                        .or_else(|_| document_from_compact_plan_text(&plan_stderr).map(planned_from_compact_document))
                        .map_err(|plan_error| {
                            format!(
                                "model did not return a valid Strut document after repair. First error: {first_error}. Repair error: {repair_error}. Plan error: {plan_error}. Response preview: {}",
                                response_preview(&plan_text)
                            )
                        })
                }
            }
        }
    }
}

async fn generate_document_with_ollama(
    prompt: &str,
    references: &[ReferenceImageInput],
) -> Result<PlannedDocument, String> {
    let client = http_client()?;
    let images = references
        .iter()
        .filter_map(|reference| data_url_payload(&reference.data_url).map(str::to_string))
        .collect::<Vec<_>>();
    let response = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&json!({
            "model": "llama3.2",
            "prompt": format!("{GENERATION_PLAN_SYSTEM_PROMPT}\nPrompt: {}", prompt_with_reference_context(prompt, references)),
            "images": images,
            "stream": false,
            "format": "json"
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let status = response.status();
    let body: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(http_error_preview(status.as_u16(), &body.to_string()));
    }

    let text = body
        .get("response")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "Ollama response did not include a response field".to_string())?;
    match parse_provider_response_document(text) {
        Ok(document) => Ok(document),
        Err(first_error) => {
            let repair_prompt = generation_plan_repair_prompt(prompt, text, &first_error);
            let repair_response = client
                .post("http://127.0.0.1:11434/api/generate")
                .json(&json!({
                    "model": "llama3.2",
                    "prompt": repair_prompt,
                    "stream": false,
                    "format": "json"
                }))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            let status = repair_response.status();
            let body: serde_json::Value = repair_response
                .json()
                .await
                .map_err(|error| error.to_string())?;
            if !status.is_success() {
                return Err(http_error_preview(status.as_u16(), &body.to_string()));
            }
            let repair_text = body
                .get("response")
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    "Ollama repair response did not include a response field".to_string()
                })?;
            match parse_provider_response_document(repair_text) {
                Ok(document) => Ok(document),
                Err(repair_error) => {
                    let plan_response = client
                        .post("http://127.0.0.1:11434/api/generate")
                        .json(&json!({
                            "model": "llama3.2",
                            "prompt": compact_plan_prompt(prompt, &repair_error),
                            "stream": false,
                            "format": "json"
                        }))
                        .send()
                        .await
                        .map_err(|error| error.to_string())?;
                    let status = plan_response.status();
                    let body: serde_json::Value = plan_response
                        .json()
                        .await
                        .map_err(|error| error.to_string())?;
                    if !status.is_success() {
                        return Err(http_error_preview(status.as_u16(), &body.to_string()));
                    }
                    let plan_text = body
                        .get("response")
                        .and_then(|value| value.as_str())
                        .ok_or_else(|| {
                            "Ollama compact plan response did not include a response field"
                                .to_string()
                        })?;
                    document_from_generation_plan_text(plan_text)
                        .or_else(|_| document_from_compact_plan_text(plan_text).map(planned_from_compact_document))
                        .map_err(|plan_error| {
                        format!(
                            "model did not return a valid Strut document after repair. First error: {first_error}. Repair error: {repair_error}. Plan error: {plan_error}. Response preview: {}",
                            response_preview(plan_text)
                        )
                    })
                }
            }
        }
    }
}

async fn openai_compatible_chat(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {
    let client = http_client()?;
    let user_content = if references.is_empty() {
        json!(prompt)
    } else {
        let mut content = vec![
            json!({"type": "text", "text": prompt_with_reference_context(prompt, references)}),
        ];
        content.extend(references.iter().map(|reference| {
            json!({
                "type": "image_url",
                "image_url": {"url": reference.data_url}
            })
        }));
        json!(content)
    };
    let response = client
        .post(format!(
            "{}/chat/completions",
            endpoint_base(&config.endpoint)
        ))
        .bearer_auth(config.api_key.as_deref().unwrap_or_default())
        .json(&json!({
            "model": config.model,
            "messages": [
                {"role": "system", "content": GENERATION_PLAN_SYSTEM_PROMPT},
                {"role": "user", "content": user_content}
            ],
            "temperature": 0.2,
            "response_format": {"type": "json_object"}
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let status = response.status();
    let body: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(http_error_preview(status.as_u16(), &body.to_string()));
    }

    body.pointer("/choices/0/message/content")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| "provider response did not include choices[0].message.content".to_string())
}

async fn anthropic_message(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {
    let client = http_client()?;
    let mut content =
        vec![json!({"type": "text", "text": prompt_with_reference_context(prompt, references)})];
    content.extend(references.iter().filter_map(|reference| {
        Some(json!({
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": image_media_type(reference),
                "data": data_url_payload(&reference.data_url)?
            }
        }))
    }));
    let response = client
        .post(format!("{}/v1/messages", endpoint_base(&config.endpoint)))
        .header("x-api-key", config.api_key.as_deref().unwrap_or_default())
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": config.model,
            "max_tokens": 8192,
            "system": GENERATION_PLAN_SYSTEM_PROMPT,
            "messages": [{"role": "user", "content": content}]
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let status = response.status();
    let body: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(http_error_preview(status.as_u16(), &body.to_string()));
    }

    body.pointer("/content/0/text")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| "Anthropic response did not include content[0].text".to_string())
}

async fn gemini_generate_content(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {
    let client = http_client()?;
    let model = config.model.trim().trim_start_matches("models/");
    let mut parts = vec![json!({
        "text": format!("{GENERATION_PLAN_SYSTEM_PROMPT}\nPrompt: {}", prompt_with_reference_context(prompt, references))
    })];
    parts.extend(references.iter().filter_map(|reference| {
        Some(json!({
            "inlineData": {
                "mimeType": image_media_type(reference),
                "data": data_url_payload(&reference.data_url)?
            }
        }))
    }));
    let response = client
        .post(format!(
            "{}/v1beta/models/{model}:generateContent?key={}",
            endpoint_base(&config.endpoint),
            config.api_key.as_deref().unwrap_or_default()
        ))
        .json(&json!({
            "contents": [{
                "parts": parts
            }],
            "generationConfig": {
                "temperature": 0.2,
                "responseMimeType": "application/json"
            }
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let status = response.status();
    let body: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(http_error_preview(status.as_u16(), &body.to_string()));
    }

    body.pointer("/candidates/0/content/parts/0/text")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            "Gemini response did not include candidates[0].content.parts[0].text".to_string()
        })
}

fn cli_assistant_text(text: &str) -> String {
    let mut collected = String::new();
    for line in text.lines() {
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            collect_text_fields(&value, &mut collected);
        }
    }
    if collected.trim().is_empty() {
        text.to_string()
    } else {
        collected
    }
}

fn collect_text_fields(value: &Value, out: &mut String) {
    match value {
        Value::String(text) => {
            out.push_str(text);
        }
        Value::Array(values) => {
            for value in values {
                collect_text_fields(value, out);
            }
        }
        Value::Object(map) => {
            if map.get("role").and_then(Value::as_str) == Some("user") {
                return;
            }
            for key in ["text", "content", "message", "delta", "output", "response"] {
                if let Some(value) = map.get(key) {
                    collect_text_fields(value, out);
                }
            }
        }
        _ => {}
    }
}

fn parse_generated_document(text: &str) -> Result<strut_core::Document, String> {
    if let Ok(document) = serde_json::from_str::<strut_core::Document>(text.trim()) {
        return validate_generated_document(document);
    }

    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        if let Ok(document) = document_from_value(&value) {
            return validate_generated_document(document);
        }
    }

    let mut last_error = None;
    for json_text in extract_json_objects(text).into_iter().rev() {
        match serde_json::from_str::<Value>(&json_text)
            .map_err(|error| error.to_string())
            .and_then(|value| document_from_value(&value))
            .and_then(validate_generated_document)
        {
            Ok(document) => return Ok(document),
            Err(error) => last_error = Some(error),
        }
    }

    if parse_character_spec(text).is_ok()
        || extract_json_objects(text)
            .iter()
            .any(|json_text| parse_character_spec(json_text).is_ok())
    {
        return Err(
            "provider returned the old preset spec format. Strut now requires a full editable document so different prompts do not collapse into the same character.".to_string(),
        );
    }

    Err(last_error.unwrap_or_else(|| "model did not return a valid Strut document".to_string()))
}

fn document_from_value(value: &Value) -> Result<strut_core::Document, String> {
    let mut document_value = value
        .get("document")
        .cloned()
        .unwrap_or_else(|| value.clone());
    normalize_generated_document_value(&mut document_value);
    serde_json::from_value::<strut_core::Document>(document_value)
        .map_err(|error| format!("model response was not a Strut document: {error}"))
}

fn normalize_generated_document_value(value: &mut Value) {
    let mut id_map = HashMap::new();
    let mut next_id = 1u128;
    normalize_generated_ids(value, &mut id_map, &mut next_id);
    fill_generated_defaults(value);
}

fn normalize_generated_ids(
    value: &mut Value,
    id_map: &mut HashMap<String, String>,
    next_id: &mut u128,
) {
    match value {
        Value::Array(values) => {
            for value in values {
                normalize_generated_ids(value, id_map, next_id);
            }
        }
        Value::Object(map) => {
            for key in ["id", "target"] {
                if key == "id"
                    && (map.contains_key("timeline") || map.contains_key("timeline_id"))
                    && !map.contains_key("kind")
                {
                    continue;
                }
                if let Some(Value::String(raw_id)) = map.get_mut(key) {
                    if !looks_like_uuid(raw_id) {
                        let strict_id = id_map.entry(raw_id.clone()).or_insert_with(|| {
                            let id = format!("00000000-0000-0000-0000-{next_id:012x}");
                            *next_id += 1;
                            id
                        });
                        *raw_id = strict_id.clone();
                    }
                }
            }
            for value in map.values_mut() {
                normalize_generated_ids(value, id_map, next_id);
            }
        }
        _ => {}
    }
}

fn fill_generated_defaults(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                fill_generated_defaults(value);
            }
        }
        Value::Object(map) => {
            if map.contains_key("artboards") {
                map.entry("timelines").or_insert_with(|| json!([]));
                map.entry("state_machines").or_insert_with(|| json!([]));
                map.entry("bindings").or_insert_with(|| json!([]));
                map.entry("events").or_insert_with(|| json!([]));
            }

            if map.contains_key("tracks") {
                if let Some(duration) = map.remove("duration") {
                    map.entry("duration_ms").or_insert(duration);
                }
            }

            if map.contains_key("keyframes") {
                if let Some(property) = map.get_mut("property") {
                    normalize_track_property(property);
                }
            }

            if (map.contains_key("time") || map.contains_key("time_ms"))
                && map.contains_key("value")
            {
                if let Some(time) = map.remove("time") {
                    map.entry("time_ms").or_insert(time);
                }
                if let Some(value) = map.get_mut("value") {
                    normalize_keyframe_value(value);
                }
                if let Some(easing) = map.get_mut("easing") {
                    normalize_easing(easing);
                } else {
                    map.insert("easing".to_string(), json!("ease_in_out"));
                }
            }

            if map.contains_key("states") {
                map.entry("name").or_insert_with(|| json!("GeneratedMoods"));
                map.entry("inputs").or_insert_with(|| json!([]));
                map.entry("transitions").or_insert_with(|| json!([]));
                if let Some(states) = map.get_mut("states") {
                    normalize_state_list(states);
                }
            }

            if map.contains_key("kind") {
                map.entry("transform")
                    .or_insert_with(|| json!(default_transform_value()));
                map.entry("style")
                    .or_insert_with(|| json!(default_style_value()));
                map.entry("shape")
                    .or_insert_with(|| json!({"type": "none"}));
                map.entry("children").or_insert_with(|| json!([]));
            }

            if let Some(style) = map.get_mut("style") {
                fill_style_defaults(style);
            }
            if let Some(transform) = map.get_mut("transform") {
                fill_transform_defaults(transform);
            }

            for value in map.values_mut() {
                fill_generated_defaults(value);
            }
        }
        _ => {}
    }
}

fn normalize_track_property(value: &mut Value) {
    let Value::String(property) = value else {
        return;
    };
    *property = match property.as_str() {
        "translate_x" | "translation_x" | "x" => "translation.x",
        "translate_y" | "translation_y" | "y" => "translation.y",
        "rotate" => "rotation",
        "scale_x" => "scale.x",
        "scale_y" => "scale.y",
        other => other,
    }
    .to_string();
}

fn normalize_keyframe_value(value: &mut Value) {
    match value {
        Value::Number(_) => {
            let number = value.clone();
            *value = json!({"type": "number", "value": number});
        }
        Value::String(text) if text.starts_with('#') => {
            let color = text.clone();
            *value = json!({"type": "color", "value": color});
        }
        Value::String(_) => {
            let text = value.clone();
            *value = json!({"type": "text", "value": text});
        }
        Value::Object(map) if !map.contains_key("type") => {
            if map.contains_key("x") && map.contains_key("y") {
                let point = value.clone();
                *value = json!({"type": "point", "value": point});
            }
        }
        _ => {}
    }
}

fn normalize_easing(value: &mut Value) {
    let Value::String(easing) = value else {
        *value = json!("ease_in_out");
        return;
    };
    *easing = match easing.as_str() {
        "easeIn" | "easeInQuad" | "ease_in_quad" | "ease-in" => "ease_in",
        "easeOut" | "easeOutQuad" | "ease_out_quad" | "ease-out" => "ease_out",
        "easeInOut" | "easeInOutQuad" | "ease_in_out_quad" | "ease-in-out" => "ease_in_out",
        "linear" => "linear",
        "ease_in" | "ease_out" | "ease_in_out" => easing.as_str(),
        _ => "ease_in_out",
    }
    .to_string();
}

fn normalize_state_list(value: &mut Value) {
    let Value::Array(states) = value else {
        return;
    };
    for state in states {
        if let Value::Object(map) = state {
            let name = map
                .get("name")
                .or_else(|| map.get("id"))
                .and_then(Value::as_str)
                .unwrap_or("idle")
                .to_lowercase()
                .replace(' ', "_");
            *state = Value::String(name);
        }
    }
}

fn fill_style_defaults(value: &mut Value) {
    let Value::Object(map) = value else {
        *value = json!(default_style_value());
        return;
    };
    normalize_none_string(map.get_mut("fill"));
    normalize_none_string(map.get_mut("stroke"));
    map.entry("fill").or_insert(Value::Null);
    map.entry("stroke").or_insert(Value::Null);
    map.entry("stroke_width").or_insert(json!(0.0));
    map.entry("opacity").or_insert(json!(1.0));
    map.entry("linecap").or_insert(json!("round"));
    map.entry("linejoin").or_insert(json!("round"));
}

fn fill_transform_defaults(value: &mut Value) {
    let Value::Object(map) = value else {
        *value = json!(default_transform_value());
        return;
    };
    map.entry("translate_x").or_insert(json!(0.0));
    map.entry("translate_y").or_insert(json!(0.0));
    map.entry("rotate").or_insert(json!(0.0));
    map.entry("scale_x").or_insert(json!(1.0));
    map.entry("scale_y").or_insert(json!(1.0));
}

fn normalize_none_string(value: Option<&mut Value>) {
    if let Some(value) = value {
        if let Value::String(text) = value {
            if text.eq_ignore_ascii_case("none") || text.eq_ignore_ascii_case("transparent") {
                *value = Value::Null;
            }
        }
    }
}

fn parse_provider_response_document(text: &str) -> Result<PlannedDocument, String> {
    document_from_generation_plan_text(text).or_else(|plan_error| {
        parse_generated_document(text).map(|document| PlannedDocument {
            document,
            summary: GenerationPlanSummary {
                subject_classification: "whole-document-fallback".to_string(),
                subject_label: "Validated whole document fallback".to_string(),
                part_names: Vec::new(),
                timeline_names: Vec::new(),
            },
            operation_count: 0,
        }).map_err(|document_error| {
            format!("plan parse failed: {plan_error}; whole-document fallback failed: {document_error}")
        })
    })
}

fn planned_from_compact_document(document: strut_core::Document) -> PlannedDocument {
    let timeline_names = document
        .timelines
        .iter()
        .map(|timeline| timeline.name.clone())
        .collect::<Vec<_>>();
    PlannedDocument {
        document,
        summary: GenerationPlanSummary {
            subject_classification: "compact-plan-fallback".to_string(),
            subject_label: "Validated compact plan fallback".to_string(),
            part_names: Vec::new(),
            timeline_names,
        },
        operation_count: 0,
    }
}

fn document_from_generation_plan_text(text: &str) -> Result<PlannedDocument, String> {
    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        if let Ok(document) = document_from_generation_plan_value(&value) {
            return Ok(document);
        }
    }

    let mut last_error = None;
    for json_text in extract_json_objects(text).into_iter().rev() {
        match serde_json::from_str::<Value>(&json_text)
            .map_err(|error| error.to_string())
            .and_then(|value| document_from_generation_plan_value(&value))
        {
            Ok(document) => return Ok(document),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| "model did not return a valid generation plan".to_string()))
}

fn document_from_generation_plan_value(value: &Value) -> Result<PlannedDocument, String> {
    let envelope_value = if value.get("plan").is_some() {
        value.clone()
    } else if let Some(plan) = value
        .get("generation_plan")
        .or_else(|| value.get("generationPlan"))
    {
        json!({
            "plan": plan,
            "operations": value.get("operations").cloned().unwrap_or_else(|| json!([]))
        })
    } else {
        return Err("generation response must include a plan object".to_string());
    };
    let envelope: GenerationPlanEnvelope = serde_json::from_value(envelope_value)
        .map_err(|error| format!("generation plan schema mismatch: {error}"))?;
    validate_generation_plan(&envelope.plan)?;
    let operations = if envelope.operations.is_empty() {
        operations_from_generation_plan(&envelope.plan)
    } else {
        envelope.operations
    };
    validate_scene_operations(&envelope.plan, &operations)?;
    let document = document_from_scene_operations(&envelope.plan, &operations)?;
    let summary = GenerationPlanSummary {
        subject_classification: envelope.plan.subject.classification.clone(),
        subject_label: envelope.plan.subject.label.clone(),
        part_names: envelope
            .plan
            .parts
            .iter()
            .map(|part| part.name.clone())
            .collect(),
        timeline_names: envelope
            .plan
            .timelines
            .iter()
            .map(|timeline| timeline.name.clone())
            .collect(),
    };

    Ok(PlannedDocument {
        document,
        summary,
        operation_count: operations.len(),
    })
}

fn validate_generation_plan(plan: &GenerationPlan) -> Result<(), String> {
    if plan.name.trim().is_empty() {
        return Err("generation plan must include a non-empty name".to_string());
    }
    if let Some(id) = &plan.id {
        if id.trim().is_empty() {
            return Err("generation plan id must not be empty".to_string());
        }
    }
    if plan.subject.classification.trim().is_empty() || plan.subject.label.trim().is_empty() {
        return Err("generation plan must classify the requested subject".to_string());
    }
    if plan.parts.len() < 5 {
        return Err("generation plan must include at least five semantic parts".to_string());
    }

    let mut part_ids = HashSet::new();
    let mut role_ids = HashSet::new();
    for role in &plan.motion_roles {
        if role.id.trim().is_empty() {
            return Err("motion role ids must not be empty".to_string());
        }
        if !role_ids.insert(role.id.as_str()) {
            return Err(format!("duplicate motion role id '{}'", role.id));
        }
        if role.purpose.trim().is_empty() {
            return Err(format!(
                "motion role '{}' must describe its purpose",
                role.id
            ));
        }
    }

    for part in &plan.parts {
        if part.id.trim().is_empty() || part.name.trim().is_empty() {
            return Err("semantic parts must include non-empty id and name".to_string());
        }
        if !part_ids.insert(part.id.as_str()) {
            return Err(format!("duplicate part id '{}'", part.id));
        }
        if part.role.trim().is_empty() {
            return Err(format!("part '{}' must include a semantic role", part.id));
        }
        validate_part_geometry(part)?;
        if !part.constraints.editable && plan.editability.locked_parts.is_empty() {
            return Err(format!(
                "part '{}' is non-editable but the plan did not list locked parts",
                part.id
            ));
        }
        for property in &part.constraints.allowed_properties {
            if !allowed_edit_property(property) {
                return Err(format!(
                    "part '{}' allows unsupported editable property '{}'",
                    part.id, property
                ));
            }
        }
        for role in &part.motion_roles {
            if !role_ids.is_empty() && !role_ids.contains(role.as_str()) {
                return Err(format!(
                    "part '{}' references missing motion role '{}'",
                    part.id, role
                ));
            }
        }
    }

    for role in &plan.motion_roles {
        for part_ref in &role.part_refs {
            if !part_ids.contains(part_ref.as_str()) {
                return Err(format!(
                    "motion role '{}' references missing part '{}'",
                    role.id, part_ref
                ));
            }
        }
    }

    if !subject_allows_mascot_anatomy(&plan.subject) {
        let mascot_parts = plan
            .parts
            .iter()
            .filter(|part| is_mascot_anatomy_name(&part.name) || is_mascot_anatomy_name(&part.id))
            .map(|part| part.name.as_str())
            .collect::<Vec<_>>();
        if !mascot_parts.is_empty() {
            return Err(format!(
                "non-mascot subject '{}' cannot use mascot-only anatomy: {}",
                plan.subject.classification,
                mascot_parts.join(", ")
            ));
        }
    }

    let states = normalized_state_set(&plan.states);
    if states.is_empty() {
        return Err("generation plan must include named states".to_string());
    }
    if !states.contains("idle") {
        return Err("generation plan must include an idle state".to_string());
    }
    let mut timeline_ids = HashSet::new();
    let mut timeline_names = HashSet::new();
    for timeline in &plan.timelines {
        if timeline.id.trim().is_empty() || timeline.name.trim().is_empty() {
            return Err("timeline plans must include id and name".to_string());
        }
        if !timeline_ids.insert(timeline.id.as_str()) {
            return Err(format!("duplicate timeline id '{}'", timeline.id));
        }
        if !timeline_names.insert(timeline.name.as_str()) {
            return Err(format!("duplicate timeline name '{}'", timeline.name));
        }
        if timeline.duration_ms == 0 {
            return Err(format!(
                "timeline '{}' duration must be greater than zero",
                timeline.name
            ));
        }
        if let Some(state) = &timeline.state {
            if !states.contains(normalized_state_name(state).as_str()) {
                return Err(format!(
                    "timeline '{}' references unknown state '{}'",
                    timeline.name, state
                ));
            }
        }
        if timeline.tracks.is_empty() {
            return Err(format!("timeline '{}' must include tracks", timeline.name));
        }
        for track in &timeline.tracks {
            if !part_ids.contains(track.target.as_str()) {
                return Err(format!(
                    "timeline '{}' track targets missing part '{}'",
                    timeline.name, track.target
                ));
            }
            if !allowed_timeline_property(&track.property) {
                return Err(format!(
                    "timeline '{}' uses unsupported property '{}'",
                    timeline.name, track.property
                ));
            }
            if track.keyframes.len() < 2 {
                return Err(format!(
                    "timeline '{}' track '{}' must include at least two keyframes",
                    timeline.name, track.target
                ));
            }
            for keyframe in &track.keyframes {
                if keyframe.time_ms > timeline.duration_ms {
                    return Err(format!(
                        "timeline '{}' keyframe at {}ms exceeds duration {}ms",
                        timeline.name, keyframe.time_ms, timeline.duration_ms
                    ));
                }
                if !keyframe.value.is_finite() {
                    return Err(format!(
                        "timeline '{}' has a non-finite keyframe value",
                        timeline.name
                    ));
                }
            }
        }
    }

    for editable_part in &plan.editability.editable_parts {
        if !part_ids.contains(editable_part.as_str()) {
            return Err(format!(
                "editability references missing editable part '{}'",
                editable_part
            ));
        }
    }
    for locked_part in &plan.editability.locked_parts {
        if !part_ids.contains(locked_part.as_str()) {
            return Err(format!(
                "editability references missing locked part '{}'",
                locked_part
            ));
        }
    }

    Ok(())
}

fn validate_part_geometry(part: &SemanticPartPlan) -> Result<(), String> {
    let geometry = &part.geometry;
    match geometry.kind.to_lowercase().as_str() {
        "rect" | "rectangle" => {
            let width = geometry.width.unwrap_or_default();
            let height = geometry.height.unwrap_or_default();
            if width <= 0.0 || height <= 0.0 || !width.is_finite() || !height.is_finite() {
                return Err(format!("part '{}' has invalid rect geometry", part.id));
            }
        }
        "ellipse" => {
            let rx = geometry
                .rx
                .or_else(|| geometry.width.map(|width| width / 2.0))
                .unwrap_or_default();
            let ry = geometry
                .ry
                .or_else(|| geometry.height.map(|height| height / 2.0))
                .unwrap_or_default();
            if rx <= 0.0 || ry <= 0.0 || !rx.is_finite() || !ry.is_finite() {
                return Err(format!("part '{}' has invalid ellipse geometry", part.id));
            }
        }
        "path" => {
            if geometry.d.as_deref().unwrap_or_default().trim().is_empty() {
                return Err(format!("part '{}' path geometry must include d", part.id));
            }
        }
        "text" => {
            let size = geometry.size.unwrap_or(24.0);
            if size <= 0.0 || !size.is_finite() {
                return Err(format!("part '{}' text geometry has invalid size", part.id));
            }
        }
        other => {
            return Err(format!(
                "part '{}' uses unsupported geometry kind '{}'",
                part.id, other
            ))
        }
    }
    Ok(())
}

fn operations_from_generation_plan(plan: &GenerationPlan) -> Vec<SceneOperation> {
    let mut operations = Vec::new();
    let child_ids = plan
        .parts
        .iter()
        .map(|part| part.id.clone())
        .collect::<Vec<_>>();
    operations.push(SceneOperation::GroupNodes {
        id: "SceneRig".to_string(),
        name: format!("{} Rig", plan.name),
        children: child_ids,
    });
    operations.extend(plan.parts.iter().map(|part| SceneOperation::CreateNode {
        id: part.id.clone(),
        name: part.name.clone(),
        kind: node_kind_from_geometry(&part.geometry).to_string(),
        parent: Some("SceneRig".to_string()),
        geometry: part.geometry.clone(),
        style: part.style.clone(),
        role: Some(part.role.clone()),
    }));
    operations.extend(plan.states.iter().map(|state| SceneOperation::AddState {
        state: normalized_state_name(state),
    }));
    for timeline in &plan.timelines {
        operations.push(SceneOperation::AddTimeline {
            id: timeline.id.clone(),
            name: timeline.name.clone(),
            state: timeline
                .state
                .as_ref()
                .map(|state| normalized_state_name(state)),
            duration_ms: timeline.duration_ms,
        });
        for track in &timeline.tracks {
            for keyframe in &track.keyframes {
                operations.push(SceneOperation::AddKeyframe {
                    timeline: timeline.id.clone(),
                    target: track.target.clone(),
                    property: normalize_motion_property(&track.property),
                    time_ms: keyframe.time_ms,
                    value: keyframe.value,
                    easing: keyframe.easing.clone(),
                });
            }
        }
    }
    for part in &plan.parts {
        if part.constraints.editable
            && !plan
                .editability
                .locked_parts
                .iter()
                .any(|id| id == &part.id)
        {
            operations.push(SceneOperation::BindProperty {
                name: format!("edit_{}_fill", semantic_token(&part.id)),
                target: part.id.clone(),
                property: "fill".to_string(),
            });
        }
    }
    operations.push(SceneOperation::EmitEvent {
        name: "generation_plan_validated".to_string(),
        description: format!(
            "{} plan converted through validated Strut operations",
            plan.subject.label
        ),
    });
    operations
}

fn validate_scene_operations(
    plan: &GenerationPlan,
    operations: &[SceneOperation],
) -> Result<(), String> {
    if operations.is_empty() {
        return Err("generation plan did not produce operations".to_string());
    }
    let plan_parts = plan
        .parts
        .iter()
        .map(|part| part.id.as_str())
        .collect::<HashSet<_>>();
    let states = normalized_state_set(&plan.states);
    let mut created_nodes = HashSet::new();
    let mut timelines: HashMap<&str, u32> = HashMap::new();
    let mut grouped_children = HashSet::new();

    for operation in operations {
        match operation {
            SceneOperation::CreateNode { id, geometry, .. } => {
                if !plan_parts.contains(id.as_str()) {
                    return Err(format!("create_node references part outside plan: '{id}'"));
                }
                if !created_nodes.insert(id.as_str()) {
                    return Err(format!("duplicate create_node id '{id}'"));
                }
                validate_plan_geometry(id, geometry)?;
            }
            SceneOperation::GroupNodes { id, children, .. } => {
                if id.trim().is_empty() {
                    return Err("group_nodes id must not be empty".to_string());
                }
                for child in children {
                    if !plan_parts.contains(child.as_str()) {
                        return Err(format!("group_nodes references missing child '{child}'"));
                    }
                    grouped_children.insert(child.as_str());
                }
            }
            SceneOperation::SetProperty {
                target, property, ..
            } => {
                if !plan_parts.contains(target.as_str()) {
                    return Err(format!("set_property references missing target '{target}'"));
                }
                if property.trim().is_empty() {
                    return Err("set_property property must not be empty".to_string());
                }
            }
            SceneOperation::AddState { state } => {
                if !states.contains(normalized_state_name(state).as_str()) {
                    return Err(format!(
                        "add_state references state outside plan: '{state}'"
                    ));
                }
            }
            SceneOperation::AddTimeline {
                id,
                duration_ms,
                state,
                ..
            } => {
                if *duration_ms == 0 {
                    return Err(format!(
                        "add_timeline '{id}' duration must be greater than zero"
                    ));
                }
                if let Some(state) = state {
                    if !states.contains(normalized_state_name(state).as_str()) {
                        return Err(format!(
                            "add_timeline '{id}' references unknown state '{state}'"
                        ));
                    }
                }
                if timelines.insert(id.as_str(), *duration_ms).is_some() {
                    return Err(format!("duplicate add_timeline id '{id}'"));
                }
            }
            SceneOperation::AddKeyframe {
                timeline,
                target,
                property,
                time_ms,
                value,
                ..
            } => {
                let Some(duration) = timelines.get(timeline.as_str()) else {
                    return Err(format!(
                        "add_keyframe references missing timeline '{timeline}'"
                    ));
                };
                if !plan_parts.contains(target.as_str()) {
                    return Err(format!("add_keyframe references missing node '{target}'"));
                }
                if *time_ms > *duration {
                    return Err(format!(
                        "add_keyframe time {time_ms} exceeds timeline '{timeline}' duration"
                    ));
                }
                if !allowed_timeline_property(property) {
                    return Err(format!(
                        "add_keyframe uses unsupported property '{property}'"
                    ));
                }
                if !value.is_finite() {
                    return Err("add_keyframe value must be finite".to_string());
                }
            }
            SceneOperation::BindProperty {
                target, property, ..
            } => {
                if !plan_parts.contains(target.as_str()) {
                    return Err(format!(
                        "bind_property references missing target '{target}'"
                    ));
                }
                if !allowed_edit_property(property) {
                    return Err(format!(
                        "bind_property uses unsupported property '{property}'"
                    ));
                }
            }
            SceneOperation::EmitEvent { name, .. } => {
                if name.trim().is_empty() {
                    return Err("emit_event name must not be empty".to_string());
                }
            }
        }
    }

    for part in &plan.parts {
        if !created_nodes.contains(part.id.as_str()) {
            return Err(format!(
                "operations did not create planned part '{}'",
                part.id
            ));
        }
        if !grouped_children.contains(part.id.as_str()) {
            return Err(format!(
                "operations did not group planned part '{}'",
                part.id
            ));
        }
    }

    Ok(())
}

fn document_from_scene_operations(
    plan: &GenerationPlan,
    operations: &[SceneOperation],
) -> Result<strut_core::Document, String> {
    let mut nodes = HashMap::<String, Value>::new();
    let mut root_group: Option<(String, String, Vec<String>)> = None;
    let mut states = Vec::<String>::new();
    let mut timeline_states = HashMap::<String, Option<String>>::new();
    let mut timelines = HashMap::<String, Value>::new();
    let mut bindings = Vec::<Value>::new();
    let mut events = Vec::<Value>::new();

    for operation in operations {
        match operation {
            SceneOperation::CreateNode {
                id,
                name,
                kind,
                geometry,
                style,
                role,
                ..
            } => {
                nodes.insert(
                    id.clone(),
                    json!({
                        "id": id,
                        "name": name,
                        "kind": normalized_node_kind(kind, geometry),
                        "role": role,
                        "transform": default_transform_value(),
                        "style": plan_style_value(style),
                        "shape": plan_geometry_shape(geometry),
                        "children": []
                    }),
                );
            }
            SceneOperation::GroupNodes { id, name, children } => {
                root_group = Some((id.clone(), name.clone(), children.clone()));
            }
            SceneOperation::SetProperty {
                target,
                property,
                value,
            } => {
                if let Some(node) = nodes.get_mut(target) {
                    set_node_property(node, property, value.clone());
                }
            }
            SceneOperation::AddState { state } => {
                push_unique(&mut states, normalized_state_name(state));
            }
            SceneOperation::AddTimeline {
                id,
                name,
                state,
                duration_ms,
            } => {
                timelines.insert(
                    id.clone(),
                    json!({
                        "id": id,
                        "name": name,
                        "duration_ms": duration_ms,
                        "tracks": []
                    }),
                );
                timeline_states.insert(id.clone(), state.clone());
            }
            SceneOperation::AddKeyframe {
                timeline,
                target,
                property,
                time_ms,
                value,
                easing,
            } => {
                if let Some(timeline_value) = timelines.get_mut(timeline) {
                    add_keyframe_to_timeline(
                        timeline_value,
                        target,
                        &normalize_motion_property(property),
                        *time_ms,
                        *value,
                        easing.as_deref().unwrap_or("ease_in_out"),
                    );
                }
            }
            SceneOperation::BindProperty {
                name,
                target,
                property,
            } => {
                bindings.push(json!({
                    "name": name,
                    "target": target,
                    "property": normalize_bind_property(property)
                }));
            }
            SceneOperation::EmitEvent { name, description } => {
                events.push(json!({
                    "name": name,
                    "description": description
                }));
            }
        }
    }

    if !states.iter().any(|state| state == "idle") {
        states.insert(0, "idle".to_string());
    }

    let root = if let Some((id, name, children)) = root_group {
        let child_values = children
            .iter()
            .filter_map(|child| nodes.remove(child))
            .collect::<Vec<_>>();
        json!({
            "id": id,
            "name": name,
            "kind": "group",
            "role": "scene_rig",
            "transform": default_transform_value(),
            "style": default_style_value(),
            "shape": {"type": "none"},
            "children": child_values
        })
    } else {
        json!({
            "id": "SceneRig",
            "name": format!("{} Rig", plan.name),
            "kind": "group",
            "role": "scene_rig",
            "transform": default_transform_value(),
            "style": default_style_value(),
            "shape": {"type": "none"},
            "children": nodes.into_values().collect::<Vec<_>>()
        })
    };

    let mut timeline_values = timelines.into_values().collect::<Vec<_>>();
    timeline_values.sort_by_key(|timeline| {
        timeline
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    });
    let transitions = timeline_values
        .iter()
        .filter_map(|timeline| {
            let id = timeline.get("id").and_then(Value::as_str)?;
            let name = timeline.get("name").and_then(Value::as_str)?;
            let state = timeline_states
                .get(id)
                .and_then(Clone::clone)
                .unwrap_or_else(|| active_state_from_timeline_name(name));
            Some(json!({
                "from": "idle",
                "to": state,
                "on": state,
                "timeline": name
            }))
        })
        .collect::<Vec<_>>();

    let document_value = json!({
        "id": plan.id.as_deref().unwrap_or("generation-plan-document"),
        "name": plan.name,
        "artboards": [{
            "id": "main-artboard",
            "name": format!("{} Artboard", semantic_label(&plan.name)),
            "width": 960,
            "height": 540,
            "nodes": [root]
        }],
        "timelines": timeline_values,
        "state_machines": [{
            "id": "motion-machine",
            "name": format!("{} Motion", semantic_label(&plan.subject.label)),
            "inputs": [{"name": "state", "kind": "enum"}],
            "states": states,
            "transitions": transitions
        }],
        "bindings": bindings,
        "events": events
    });

    document_from_value(&document_value).and_then(validate_generated_document)
}

fn validate_plan_geometry(id: &str, geometry: &PlanGeometry) -> Result<(), String> {
    let part = SemanticPartPlan {
        id: id.to_string(),
        name: id.to_string(),
        role: "operation".to_string(),
        geometry: geometry.clone(),
        style: PlanStyle::default(),
        motion_roles: Vec::new(),
        constraints: EditabilityConstraint::default(),
    };
    validate_part_geometry(&part)
}

fn plan_style_value(style: &PlanStyle) -> Value {
    let fill = style.fill.as_deref().unwrap_or("#f6f0df");
    json!({
        "fill": if fill.eq_ignore_ascii_case("none") || fill.eq_ignore_ascii_case("transparent") { Value::Null } else { json!(fill) },
        "stroke": style.stroke.as_deref().unwrap_or("#25221d"),
        "stroke_width": style.stroke_width.unwrap_or(5.0),
        "opacity": style.opacity.unwrap_or(1.0),
        "linecap": "round",
        "linejoin": "round"
    })
}

fn plan_geometry_shape(geometry: &PlanGeometry) -> Value {
    match geometry.kind.to_lowercase().as_str() {
        "rect" | "rectangle" => json!({
            "type": "rect",
            "x": geometry.x.unwrap_or(420.0),
            "y": geometry.y.unwrap_or(220.0),
            "width": geometry.width.unwrap_or(80.0),
            "height": geometry.height.unwrap_or(80.0),
            "rx": geometry.rx.unwrap_or(12.0)
        }),
        "path" => json!({
            "type": "path",
            "d": geometry.d.as_deref().unwrap_or("M420 240 C460 210 500 270 540 240")
        }),
        "text" => json!({
            "type": "text",
            "x": geometry.x.unwrap_or(420.0),
            "y": geometry.y.unwrap_or(280.0),
            "value": geometry.value.as_deref().unwrap_or("Strut"),
            "size": geometry.size.unwrap_or(28.0)
        }),
        _ => json!({
            "type": "ellipse",
            "cx": geometry.cx.or(geometry.x).unwrap_or(480.0),
            "cy": geometry.cy.or(geometry.y).unwrap_or(270.0),
            "rx": geometry.rx.or_else(|| geometry.width.map(|width| width / 2.0)).unwrap_or(42.0),
            "ry": geometry.ry.or_else(|| geometry.height.map(|height| height / 2.0)).unwrap_or(42.0)
        }),
    }
}

fn set_node_property(node: &mut Value, property: &str, value: Value) {
    let Some(map) = node.as_object_mut() else {
        return;
    };
    let normalized = normalize_bind_property(property);
    if let Some(property_name) = normalized.strip_prefix("style.") {
        if let Some(style) = map.get_mut("style").and_then(Value::as_object_mut) {
            style.insert(property_name.to_string(), value);
        }
    } else if let Some(property_name) = normalized.strip_prefix("transform.") {
        if let Some(transform) = map.get_mut("transform").and_then(Value::as_object_mut) {
            transform.insert(property_name.to_string(), value);
        }
    }
}

fn add_keyframe_to_timeline(
    timeline: &mut Value,
    target: &str,
    property: &str,
    time_ms: u32,
    value: f64,
    easing: &str,
) {
    let Some(tracks) = timeline.get_mut("tracks").and_then(Value::as_array_mut) else {
        return;
    };
    let normalized_property = normalize_motion_property(property);
    if let Some(track) = tracks.iter_mut().find(|track| {
        track.get("target").and_then(Value::as_str) == Some(target)
            && track.get("property").and_then(Value::as_str) == Some(normalized_property.as_str())
    }) {
        if let Some(keyframes) = track.get_mut("keyframes").and_then(Value::as_array_mut) {
            keyframes.push(keyframe_value(time_ms, value, easing));
        }
        return;
    }
    tracks.push(json!({
        "target": target,
        "property": normalized_property,
        "keyframes": [keyframe_value(time_ms, value, easing)]
    }));
}

fn keyframe_value(time_ms: u32, value: f64, easing: &str) -> Value {
    json!({
        "time_ms": time_ms,
        "value": {"type": "number", "value": value},
        "easing": normalized_easing_name(easing)
    })
}

fn normalized_node_kind(kind: &str, geometry: &PlanGeometry) -> &'static str {
    match kind.to_lowercase().as_str() {
        "rect" | "rectangle" => "rect",
        "path" => "path",
        "text" => "text",
        "group" => "group",
        "ellipse" => "ellipse",
        _ => node_kind_from_geometry(geometry),
    }
}

fn node_kind_from_geometry(geometry: &PlanGeometry) -> &'static str {
    match geometry.kind.to_lowercase().as_str() {
        "rect" | "rectangle" => "rect",
        "path" => "path",
        "text" => "text",
        _ => "ellipse",
    }
}

fn normalized_state_set(states: &[String]) -> HashSet<String> {
    states
        .iter()
        .map(|state| normalized_state_name(state))
        .collect()
}

fn normalized_state_name(state: &str) -> String {
    let normalized = semantic_token(state).to_lowercase();
    if normalized.is_empty() {
        "idle".to_string()
    } else {
        normalized
    }
}

fn active_state_from_timeline_name(name: &str) -> String {
    match name {
        "idle_float" => "idle".to_string(),
        other => normalized_state_name(other),
    }
}

fn normalize_motion_property(property: &str) -> String {
    match property {
        "translate_x" | "translation_x" | "x" | "transform.translate_x" => "translation.x",
        "translate_y" | "translation_y" | "y" | "transform.translate_y" => "translation.y",
        "rotate" | "transform.rotate" => "rotation",
        "scale_x" | "transform.scale_x" => "scale.x",
        "scale_y" | "transform.scale_y" => "scale.y",
        "style.opacity" => "opacity",
        other => other,
    }
    .to_string()
}

fn normalize_bind_property(property: &str) -> String {
    match property {
        "fill" => "style.fill",
        "stroke" => "style.stroke",
        "opacity" => "style.opacity",
        "translate_x" | "translation.x" => "transform.translate_x",
        "translate_y" | "translation.y" => "transform.translate_y",
        "rotation" => "transform.rotate",
        other => other,
    }
    .to_string()
}

fn normalized_easing_name(easing: &str) -> &'static str {
    match easing {
        "linear" => "linear",
        "ease_in" | "easeIn" | "ease-in" => "ease_in",
        "ease_out" | "easeOut" | "ease-out" => "ease_out",
        _ => "ease_in_out",
    }
}

fn allowed_timeline_property(property: &str) -> bool {
    matches!(
        normalize_motion_property(property).as_str(),
        "translation.x"
            | "translation.y"
            | "rotation"
            | "scale"
            | "scale.x"
            | "scale.y"
            | "opacity"
    )
}

fn allowed_edit_property(property: &str) -> bool {
    matches!(
        normalize_bind_property(property).as_str(),
        "style.fill"
            | "style.stroke"
            | "style.opacity"
            | "transform.translate_x"
            | "transform.translate_y"
            | "transform.rotate"
            | "transform.scale"
            | "fill"
            | "stroke"
            | "opacity"
    )
}

fn subject_allows_mascot_anatomy(subject: &SubjectPlan) -> bool {
    let classification = subject.classification.to_lowercase();
    let label = subject.label.to_lowercase();
    [
        "mascot",
        "character",
        "avatar",
        "person",
        "human",
        "creature",
    ]
    .iter()
    .any(|word| classification.contains(word) || label.contains(word))
}

fn is_mascot_anatomy_name(value: &str) -> bool {
    let token = semantic_token(value).to_lowercase();
    matches!(
        token.as_str(),
        "body"
            | "head"
            | "face"
            | "eyes"
            | "eye"
            | "arms"
            | "arm"
            | "leftarm"
            | "rightarm"
            | "legs"
            | "leg"
            | "leftleg"
            | "rightleg"
            | "torso"
            | "mouth"
            | "smile"
    )
}

fn semantic_token(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>()
}

fn semantic_label(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn document_from_compact_plan_text(text: &str) -> Result<strut_core::Document, String> {
    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        if let Ok(document) = document_from_compact_plan_value(&value) {
            return validate_generated_document(document);
        }
    }

    let mut last_error = None;
    for json_text in extract_json_objects(text).into_iter().rev() {
        match serde_json::from_str::<Value>(&json_text)
            .map_err(|error| error.to_string())
            .and_then(|value| document_from_compact_plan_value(&value))
            .and_then(validate_generated_document)
        {
            Ok(document) => return Ok(document),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| "model did not return a valid compact Strut plan".to_string()))
}

fn document_from_compact_plan_value(value: &Value) -> Result<strut_core::Document, String> {
    let plan = value.get("plan").unwrap_or(value);
    let name = plan
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Generated Scene");
    let parts = plan
        .get("parts")
        .or_else(|| plan.get("nodes"))
        .or_else(|| plan.get("layers"))
        .and_then(Value::as_array)
        .ok_or_else(|| "compact plan must include a parts array".to_string())?;

    if parts.len() < 5 {
        return Err("compact plan must include at least five visual parts".to_string());
    }

    let children = parts
        .iter()
        .take(16)
        .enumerate()
        .map(|(index, part)| compact_part_node(index, part))
        .collect::<Vec<_>>();
    let document_value = json!({
        "id": "compact-document",
        "name": name,
        "artboards": [{
            "id": "main-artboard",
            "name": "Main",
            "width": 960,
            "height": 540,
            "nodes": [{
                "id": "root",
                "name": format!("{name} Rig"),
                "kind": "group",
                "transform": default_transform_value(),
                "style": default_style_value(),
                "shape": {"type": "none"},
                "children": children
            }]
        }],
        "timelines": compact_timelines_value(),
        "state_machines": [{
            "id": "moods",
            "name": "GeneratedMoods",
            "inputs": [{"name": "mood", "kind": "enum"}, {"name": "complete", "kind": "trigger"}],
            "states": ["idle", "float", "wave", "blink", "scan", "celebrate", "sleep"],
            "transitions": [
                {"from": "idle", "to": "float", "on": "mood", "timeline": "idle_float"},
                {"from": "idle", "to": "wave", "on": "mood", "timeline": "wave"},
                {"from": "idle", "to": "blink", "on": "mood", "timeline": "blink"},
                {"from": "idle", "to": "scan", "on": "mood", "timeline": "scan"},
                {"from": "idle", "to": "celebrate", "on": "complete", "timeline": "celebrate"}
            ]
        }],
        "bindings": [{"name": "mood", "target": "root", "property": "state"}],
        "events": [{"name": "ready", "description": "Generated from compact Strut plan"}]
    });

    document_from_value(&document_value)
}

fn compact_part_node(index: usize, part: &Value) -> Value {
    let name = part
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Part{}", index + 1));
    let kind = part
        .get("kind")
        .or_else(|| part.get("shape"))
        .and_then(Value::as_str)
        .unwrap_or("ellipse")
        .to_lowercase();
    let x = number_field(part, &["x", "cx"], 420.0 + (index as f64 * 18.0));
    let y = number_field(part, &["y", "cy"], 220.0 + (index as f64 * 14.0));
    let width = number_field(part, &["width", "w"], 80.0);
    let height = number_field(part, &["height", "h"], 80.0);
    let fill = string_field(part, &["fill", "color"]).unwrap_or_else(|| "#f6f0df".to_string());
    let stroke =
        string_field(part, &["stroke", "outline"]).unwrap_or_else(|| "#25221d".to_string());
    let stroke_width = number_field(part, &["stroke_width", "strokeWidth"], 5.0);
    let shape = match kind.as_str() {
        "rect" | "rectangle" => json!({
            "type": "rect",
            "x": x,
            "y": y,
            "width": width,
            "height": height,
            "rx": number_field(part, &["rx", "radius"], 18.0)
        }),
        "path" => json!({
            "type": "path",
            "d": string_field(part, &["d", "path"]).unwrap_or_else(|| {
                format!("M{x} {y} C{} {} {} {} {} {}", x + width * 0.5, y - height * 0.5, x + width, y + height * 0.5, x, y + height)
            })
        }),
        "text" => json!({
            "type": "text",
            "x": x,
            "y": y,
            "value": string_field(part, &["value", "text"]).unwrap_or(name.clone()),
            "size": number_field(part, &["size", "font_size"], 24.0)
        }),
        _ => json!({
            "type": "ellipse",
            "cx": x,
            "cy": y,
            "rx": number_field(part, &["rx"], width / 2.0),
            "ry": number_field(part, &["ry"], height / 2.0)
        }),
    };
    let node_kind = match kind.as_str() {
        "rect" | "rectangle" => "rect",
        "path" => "path",
        "text" => "text",
        _ => "ellipse",
    };

    json!({
        "id": format!("part-{index}"),
        "name": name,
        "kind": node_kind,
        "transform": default_transform_value(),
        "style": {
            "fill": if node_kind == "path" && fill.eq_ignore_ascii_case("none") { Value::Null } else { json!(fill) },
            "stroke": stroke,
            "stroke_width": stroke_width,
            "opacity": number_field(part, &["opacity"], 1.0),
            "linecap": "round",
            "linejoin": "round"
        },
        "shape": shape,
        "children": []
    })
}

fn compact_timelines_value() -> Value {
    json!([
        {"id": "timeline-idle-float", "name": "idle_float", "duration_ms": 1400, "tracks": [{"target": "root", "property": "translate_y", "keyframes": [{"time_ms": 0, "value": {"type": "number", "value": 0}, "easing": "ease_in_out"}, {"time_ms": 700, "value": {"type": "number", "value": -8}, "easing": "ease_out"}, {"time_ms": 1400, "value": {"type": "number", "value": 0}, "easing": "ease_in_out"}]}]},
        {"id": "timeline-wave", "name": "wave", "duration_ms": 960, "tracks": [{"target": "root", "property": "rotation", "keyframes": [{"time_ms": 0, "value": {"type": "number", "value": -2}, "easing": "ease_out"}, {"time_ms": 480, "value": {"type": "number", "value": 3}, "easing": "ease_in_out"}, {"time_ms": 960, "value": {"type": "number", "value": -2}, "easing": "ease_in"}]}]},
        {"id": "timeline-blink", "name": "blink", "duration_ms": 420, "tracks": [{"target": "root", "property": "scale.y", "keyframes": [{"time_ms": 0, "value": {"type": "number", "value": 1}, "easing": "ease_out"}, {"time_ms": 210, "value": {"type": "number", "value": 0.985}, "easing": "ease_in_out"}, {"time_ms": 420, "value": {"type": "number", "value": 1}, "easing": "ease_out"}]}]},
        {"id": "timeline-scan", "name": "scan", "duration_ms": 1200, "tracks": [{"target": "root", "property": "translate_x", "keyframes": [{"time_ms": 0, "value": {"type": "number", "value": -5}, "easing": "ease_out"}, {"time_ms": 600, "value": {"type": "number", "value": 5}, "easing": "ease_in_out"}, {"time_ms": 1200, "value": {"type": "number", "value": 0}, "easing": "ease_in"}]}]},
        {"id": "timeline-celebrate", "name": "celebrate", "duration_ms": 1180, "tracks": [{"target": "root", "property": "scale.x", "keyframes": [{"time_ms": 0, "value": {"type": "number", "value": 1}, "easing": "ease_out"}, {"time_ms": 560, "value": {"type": "number", "value": 1.045}, "easing": "ease_in_out"}, {"time_ms": 1180, "value": {"type": "number", "value": 1}, "easing": "ease_in"}]}]}
    ])
}

fn number_field(value: &Value, keys: &[&str], fallback: f64) -> f64 {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_f64))
        .unwrap_or(fallback)
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .map(str::to_string)
}

fn default_style_value() -> Value {
    json!({
        "fill": null,
        "stroke": null,
        "stroke_width": 0.0,
        "opacity": 1.0,
        "linecap": "round",
        "linejoin": "round"
    })
}

fn default_transform_value() -> Value {
    json!({
        "translate_x": 0.0,
        "translate_y": 0.0,
        "rotate": 0.0,
        "scale_x": 1.0,
        "scale_y": 1.0
    })
}

fn looks_like_uuid(value: &str) -> bool {
    value.len() == 36
        && value
            .chars()
            .enumerate()
            .all(|(index, character)| match index {
                8 | 13 | 18 | 23 => character == '-',
                _ => character.is_ascii_hexdigit(),
            })
}

fn validate_generated_document(
    document: strut_core::Document,
) -> Result<strut_core::Document, String> {
    strut_format::validate_document(&document)
        .map_err(|error| format!("generated Strut document failed validation: {error}"))?;
    if document.timelines.is_empty() {
        return Err("generated Strut document must include timelines".to_string());
    }
    if document.state_machines.is_empty() {
        return Err("generated Strut document must include a state machine".to_string());
    }
    if document
        .state_machines
        .iter()
        .all(|machine| !machine.states.iter().any(|state| state == "idle"))
    {
        return Err("generated Strut document must include an idle state".to_string());
    }
    if count_document_nodes(&document) < 6 {
        return Err(
            "generated Strut document must contain at least six editable nodes".to_string(),
        );
    }
    Ok(document)
}

fn count_document_nodes(document: &strut_core::Document) -> usize {
    document
        .artboards
        .iter()
        .map(|artboard| count_nodes(&artboard.nodes))
        .sum()
}

fn count_nodes(nodes: &[strut_core::Node]) -> usize {
    nodes
        .iter()
        .map(|node| 1 + count_nodes(&node.children))
        .sum()
}

fn parse_character_spec(text: &str) -> Result<strut_core::CharacterSpec, String> {
    if let Ok(spec) = serde_json::from_str::<strut_core::CharacterSpec>(text.trim()) {
        return Ok(spec);
    }

    for json_text in extract_json_objects(text).into_iter().rev() {
        if let Ok(spec) = serde_json::from_str::<strut_core::CharacterSpec>(&json_text) {
            return Ok(spec);
        }
    }

    Err("model did not return a valid Strut character spec".to_string())
}

fn extract_json_objects(text: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, character) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = start.take() {
                        objects.push(text[start..=index].to_string());
                    }
                }
            }
            _ => {}
        }
    }

    objects
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            studio_status,
            default_project_location,
            create_project,
            save_project_snapshot,
            load_project_snapshot,
            validate_scene_document,
            validate_generation_plan_batch,
            open_project_folder,
            generate_character,
            local_agent_adapters,
            test_local_adapter,
            test_byok_provider,
            save_byok_provider
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_layer_names<'a>(nodes: &'a [strut_core::Node], names: &mut Vec<&'a str>) {
        for node in nodes {
            names.push(node.name.as_str());
            collect_layer_names(&node.children, names);
        }
    }

    fn phase3_part(id: &str, name: &str, role: &str, geometry: Value) -> Value {
        json!({
            "id": id,
            "name": name,
            "role": role,
            "geometry": geometry,
            "style": {"fill": "#f6f0df", "stroke": "#25221d", "strokeWidth": 5, "opacity": 1},
            "motionRoles": ["primary"],
            "constraints": {"editable": true, "allowedProperties": ["fill", "translation.x", "translation.y", "rotation", "opacity"]}
        })
    }

    fn phase3_plan_text(
        classification: &str,
        label: &str,
        parts: Vec<Value>,
        state: &str,
        target: &str,
    ) -> String {
        json!({
            "plan": {
                "id": format!("{classification}-plan"),
                "name": format!("{label} Motion"),
                "subject": {"classification": classification, "label": label},
                "parts": parts,
                "motionRoles": [{"id": "primary", "purpose": "calm subject motion", "partRefs": [target]}],
                "states": ["idle", state],
                "timelines": [{
                    "id": format!("{state}-timeline"),
                    "name": state,
                    "state": state,
                    "durationMs": 1200,
                    "tracks": [{
                        "target": target,
                        "property": "translation.y",
                        "keyframes": [
                            {"timeMs": 0, "value": 0, "easing": "ease_in_out"},
                            {"timeMs": 600, "value": -8, "easing": "ease_out"},
                            {"timeMs": 1200, "value": 0, "easing": "ease_in_out"}
                        ]
                    }]
                }],
                "editability": {"editableParts": [target], "lockedParts": [], "notes": ["fixture"]}
            },
            "operations": []
        })
        .to_string()
    }

    fn semantic_layer_names(document: &strut_core::Document) -> Vec<String> {
        let mut names = Vec::new();
        collect_layer_names(&document.artboards[0].nodes, &mut names);
        names.into_iter().map(str::to_string).collect()
    }

    fn sprite_python_fixture(name: &str) -> &'static str {
        match name {
            "dice" => include_str!("../../../../packages/strut-python/fixtures/dice.plan.json"),
            "logo" => include_str!("../../../../packages/strut-python/fixtures/logo.plan.json"),
            "loader" => include_str!("../../../../packages/strut-python/fixtures/loader.plan.json"),
            "mascot" => include_str!("../../../../packages/strut-python/fixtures/mascot.plan.json"),
            "ui" => include_str!("../../../../packages/strut-python/fixtures/ui.plan.json"),
            "icon" => include_str!("../../../../packages/strut-python/fixtures/icon.plan.json"),
            _ => panic!("unknown sprite-python fixture"),
        }
    }

    #[test]
    fn sprite_python_fixtures_validate_through_generation_plan_path() {
        for (fixture, classification, required_layers, forbidden_layers) in [
            (
                "dice",
                "dice",
                vec!["DieBody", "FrontFace", "Pips"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
            (
                "logo",
                "logo",
                vec!["PrimaryMark", "Wordmark", "AccentStroke"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
            (
                "loader",
                "loader",
                vec!["Track", "ActiveSegment", "ProgressSweep"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
            ("mascot", "mascot", vec!["Body", "Head", "Eyes"], vec![]),
            (
                "ui",
                "ui",
                vec!["ButtonSurface", "ButtonLabel", "FocusRing"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
            (
                "icon",
                "badge",
                vec!["BadgePlate", "InnerShield", "StatusDot"],
                vec!["Body", "Head", "Eyes", "Arms", "Face", "Smile"],
            ),
        ] {
            let planned = document_from_generation_plan_text(sprite_python_fixture(fixture))
                .expect("sprite-python fixture should validate");
            let names = semantic_layer_names(&planned.document);

            assert_eq!(planned.summary.subject_classification, classification);
            assert!(planned.operation_count >= 10);
            for required in required_layers {
                assert!(
                    names.iter().any(|name| name == required),
                    "{fixture} missing expected layer {required}"
                );
            }
            for forbidden in forbidden_layers {
                assert!(
                    names.iter().all(|name| name != forbidden),
                    "{fixture} unexpectedly emitted mascot-only layer {forbidden}"
                );
            }
        }
    }

    fn temp_project_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("strut-{name}-{}", unix_timestamp()))
    }

    fn write_project_manifest(root: &Path, name: &str, main_scene: &str) {
        fs::write(
            root.join(PROJECT_MANIFEST_FILE),
            serde_json::to_string_pretty(&json!({
                "name": name,
                "mainScene": main_scene
            }))
            .expect("manifest json"),
        )
        .expect("manifest write");
    }

    fn write_project_scene(root: &Path, scene: &str, document: &strut_core::Document) {
        let scene_path = root.join(scene);
        fs::create_dir_all(scene_path.parent().expect("scene parent")).expect("scene dir");
        strut_format::write_strut_file(
            scene_path,
            &strut_format::StrutPackage::current(document.clone()),
        )
        .expect("scene write");
    }

    fn valid_test_batch(document: &strut_core::Document) -> OperationBatchRecord {
        OperationBatchRecord {
            id: "batch-manual-fill".to_string(),
            source_type: "manual".to_string(),
            status: "applied".to_string(),
            validation_result: OperationValidationResult {
                ok: true,
                message: "validated test operation".to_string(),
                validator: "strut-studio-rust".to_string(),
                validated_at: "1".to_string(),
            },
            document_revision_id: document_revision_id(document),
            previous_document_revision_id: Some("rev-before".to_string()),
            prompt: Some("make the button warmer".to_string()),
            source_metadata: Some(json!({"chatMessageId": "message-1"})),
            operations: vec![json!({
                "id": "op-fill",
                "type": "set_property",
                "targetId": document.artboards[0].nodes[0].id.to_string(),
                "property": "style.fill",
                "value": "#d8f5e3"
            })],
            created_at: "1".to_string(),
            updated_at: "2".to_string(),
            applied_at: Some("2".to_string()),
            rejected_at: None,
        }
    }

    fn generated_reference_test_batch(
        document: &strut_core::Document,
        operations: Vec<Value>,
    ) -> OperationBatchRecord {
        let mut batch = valid_test_batch(document);
        batch.id = "batch-generated-refs".to_string();
        batch.source_type = "sprite-python".to_string();
        batch.prompt = Some("generated reference validation".to_string());
        batch.source_metadata = Some(json!({"test": "generated-refs"}));
        batch.operations = operations;
        batch
    }

    fn generated_rect_node(id: &str) -> Value {
        json!({
            "id": id,
            "type": "create_node",
            "name": id,
            "kind": "rect",
            "geometry": {"kind": "rect", "x": 10, "y": 10, "width": 24, "height": 24, "rx": 4},
            "style": {"fill": "#ffffff", "stroke": "#111827", "strokeWidth": 2, "opacity": 1}
        })
    }

    fn generated_timeline(id: &str, name: &str) -> Value {
        json!({
            "id": id,
            "type": "add_timeline",
            "name": name,
            "state": "hover",
            "duration_ms": 180
        })
    }

    fn generated_keyframe(timeline: &str, target: &str) -> Value {
        json!({
            "id": "op-generated-keyframe",
            "type": "add_keyframe",
            "timeline": timeline,
            "target": target,
            "property": "translation.y",
            "time_ms": 0,
            "value": 0
        })
    }

    fn unrelated_operation_id(id: &str) -> Value {
        json!({
            "id": id,
            "type": "emit_event",
            "name": "submit",
            "description": "unrelated operation id must not become a node or timeline ref"
        })
    }

    #[test]
    fn project_snapshot_saves_loads_validated_scene_and_operation_batches() {
        let root = temp_project_root("snapshot");
        let document = strut_core::Document::sample_login_button();
        let batch = valid_test_batch(&document);
        let selection = PersistedSelectionState {
            active_state: "hover".to_string(),
            selected_node_id: Some(document.artboards[0].nodes[0].id.to_string()),
            layer_ui: json!({"selected": {"visible": true, "locked": false}}),
        };

        let saved = save_project_snapshot(
            root.display().to_string(),
            "Snapshot Project".to_string(),
            document.clone(),
            vec![batch.clone()],
            Some(selection.clone()),
        )
        .expect("snapshot should save");
        assert!(PathBuf::from(&saved.project.path)
            .join(MAIN_SCENE_FILE)
            .exists());
        assert!(PathBuf::from(&saved.project.path)
            .join(OPERATION_BATCHES_FILE)
            .exists());

        let loaded =
            load_project_snapshot(root.display().to_string()).expect("snapshot should load");
        assert_eq!(loaded.document.name, document.name);
        assert_eq!(loaded.operation_batches, vec![batch]);
        assert_eq!(
            loaded.selection.expect("selection").selected_node_id,
            selection.selected_node_id
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn operation_payload_validation_rejects_malformed_targets_properties_and_empty_batches() {
        let document = strut_core::Document::sample_login_button();
        let mut unsupported_type = valid_test_batch(&document);
        unsupported_type.operations = vec![json!({"id": "op-delete", "type": "delete_node"})];
        let error = validate_operation_batches(&[unsupported_type], &document)
            .expect_err("unsupported operation type rejects");
        assert!(error.contains("unsupported operation type"));

        let mut missing_target = valid_test_batch(&document);
        missing_target.operations[0]["targetId"] = json!("00000000-0000-0000-0000-000000009999");
        let error = validate_operation_batches(&[missing_target], &document)
            .expect_err("missing target rejects");
        assert!(error.contains("unknown node id"));

        let mut unsupported_property = valid_test_batch(&document);
        unsupported_property.operations[0]["property"] = json!("style.__proto__");
        let error = validate_operation_batches(&[unsupported_property], &document)
            .expect_err("unsafe property rejects");
        assert!(error.contains("unsupported set_property path"));

        let mut invalid_value = valid_test_batch(&document);
        invalid_value.operations[0]["value"] = json!({"unexpected": "object"});
        let error = validate_operation_batches(&[invalid_value], &document)
            .expect_err("invalid property value rejects");
        assert!(error.contains("invalid value"));

        let mut empty_applied = valid_test_batch(&document);
        empty_applied.operations = Vec::new();
        let error = validate_operation_batches(&[empty_applied], &document)
            .expect_err("empty applied batch rejects");
        assert!(error.contains("no meaningful operations"));
    }

    #[test]
    fn generated_references_reject_unrelated_operation_ids() {
        let document = strut_core::Document::sample_login_button();
        let existing_node = document.artboards[0].nodes[0].id.to_string();

        let add_keyframe_target = generated_reference_test_batch(
            &document,
            vec![
                unrelated_operation_id("FakeGeneratedNode"),
                generated_timeline("GeneratedTimeline", "generated-timeline"),
                generated_keyframe("GeneratedTimeline", "FakeGeneratedNode"),
            ],
        );
        let error = validate_operation_batches(&[add_keyframe_target], &document)
            .expect_err("unrelated operation id must not become a keyframe target");
        assert!(error.contains("targets unknown node 'FakeGeneratedNode'"));

        let add_keyframe_timeline = generated_reference_test_batch(
            &document,
            vec![
                unrelated_operation_id("FakeGeneratedTimeline"),
                generated_keyframe("FakeGeneratedTimeline", &existing_node),
            ],
        );
        let error = validate_operation_batches(&[add_keyframe_timeline], &document)
            .expect_err("unrelated operation id must not become a keyframe timeline");
        assert!(error.contains("unknown timeline 'FakeGeneratedTimeline'"));

        let bind_property_target = generated_reference_test_batch(
            &document,
            vec![
                unrelated_operation_id("FakeGeneratedNode"),
                json!({
                    "id": "op-bind-fake",
                    "type": "bind_property",
                    "name": "fake_binding",
                    "target": "FakeGeneratedNode",
                    "property": "fill"
                }),
            ],
        );
        let error = validate_operation_batches(&[bind_property_target], &document)
            .expect_err("unrelated operation id must not become a bind target");
        assert!(error.contains("targets unknown node 'FakeGeneratedNode'"));

        let group_nodes_child = generated_reference_test_batch(
            &document,
            vec![
                unrelated_operation_id("FakeGeneratedNode"),
                json!({
                    "id": "op-group-fake",
                    "type": "group_nodes",
                    "name": "Fake Group",
                    "children": ["FakeGeneratedNode"]
                }),
            ],
        );
        let error = validate_operation_batches(&[group_nodes_child], &document)
            .expect_err("unrelated operation id must not become a group child");
        assert!(error.contains("unknown child 'FakeGeneratedNode'"));
    }

    #[test]
    fn generated_references_accept_create_node_and_add_timeline_refs() {
        let document = strut_core::Document::sample_login_button();
        let batch = generated_reference_test_batch(
            &document,
            vec![
                generated_rect_node("GeneratedNode"),
                generated_timeline("GeneratedTimeline", "Generated Timeline"),
                json!({
                    "id": "op-group-generated",
                    "type": "group_nodes",
                    "name": "Generated Group",
                    "children": ["GeneratedNode"]
                }),
                generated_keyframe("GeneratedTimeline", "GeneratedNode"),
                generated_keyframe("Generated Timeline", "GeneratedNode"),
                json!({
                    "id": "op-bind-generated",
                    "type": "bind_property",
                    "name": "generated_fill",
                    "target": "GeneratedNode",
                    "property": "fill"
                }),
            ],
        );

        validate_operation_batches(&[batch], &document)
            .expect("create_node ids and add_timeline ids/names are valid generated refs");
    }

    #[test]
    fn replacement_operation_documents_are_validated_before_persistence() {
        let document = strut_core::Document::sample_login_button();
        let mut invalid_document = document.clone();
        invalid_document.artboards.clear();

        let mut invalid_replacement = valid_test_batch(&document);
        invalid_replacement.operations = vec![json!({
            "id": "op-replace-invalid",
            "type": "replace_document",
            "previousDocument": document,
            "nextDocument": invalid_document
        })];
        let error = validate_operation_batches(&[invalid_replacement], &document)
            .expect_err("invalid replacement document rejects");
        assert!(error.contains("replacement document"));
        assert!(error.contains("artboard"));

        let mut valid_replacement = valid_test_batch(&strut_core::Document::sample_login_button());
        let previous_document = strut_core::Document::sample_login_button();
        let next_document = strut_core::Document::sample_minimal_bot();
        valid_replacement.operations = vec![json!({
            "id": "op-replace-valid",
            "type": "replace_document",
            "previousDocument": previous_document,
            "nextDocument": next_document
        })];
        validate_operation_batches(
            &[valid_replacement],
            &strut_core::Document::sample_minimal_bot(),
        )
        .expect("valid replacement document accepts");
    }

    #[test]
    fn sprite_python_generated_operations_persist_after_rust_payload_validation() {
        let root = temp_project_root("sprite-generated-persist");
        let validated = validate_generation_plan_batch(
            sprite_python_fixture("dice").to_string(),
            "sprite-python".to_string(),
            Some("rolling dice".to_string()),
        )
        .expect("sprite-python fixture validates");

        save_project_snapshot(
            root.display().to_string(),
            "Sprite Persist".to_string(),
            validated.document.clone(),
            vec![validated.batch.clone()],
            None,
        )
        .expect("validated sprite-python operations persist");

        let loaded = load_project_snapshot(root.display().to_string())
            .expect("persisted sprite-python project loads");
        assert_eq!(loaded.operation_batches, vec![validated.batch]);
        assert_eq!(loaded.document.name, "Rolling Dice Motion");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_documents_and_batches_are_rejected_before_persistence() {
        let root = temp_project_root("invalid");
        let mut document = strut_core::Document::sample_login_button();
        document.artboards.clear();
        let validation = validate_scene_document(document.clone());
        assert!(!validation.ok);
        assert!(validation.message.contains("artboard"));

        let error = save_project_snapshot(
            root.display().to_string(),
            "Invalid Project".to_string(),
            document,
            Vec::new(),
            None,
        )
        .expect_err("bad document should reject");
        assert!(error.contains("artboard"));

        let valid_document = strut_core::Document::sample_login_button();
        let mut batch = valid_test_batch(&valid_document);
        batch.source_type = "python".to_string();
        let error = save_project_snapshot(
            root.display().to_string(),
            "Invalid Project".to_string(),
            valid_document,
            vec![batch],
            None,
        )
        .expect_err("bad batch should reject");
        assert!(error.contains("unsupported source type"));
    }

    #[test]
    fn legacy_generated_local_state_document_json_loads_for_compatibility() {
        let root = temp_project_root("legacy");
        fs::create_dir_all(root.join("scenes")).expect("scenes dir");
        let document = strut_core::Document::sample_login_button();
        fs::write(
            root.join(LEGACY_STARTER_SCENE_FILE),
            serde_json::to_string_pretty(&document).expect("document json"),
        )
        .expect("legacy scene");
        fs::write(
            root.join(PROJECT_MANIFEST_FILE),
            serde_json::to_string_pretty(&json!({
                "name": "Legacy Project",
                "mainScene": LEGACY_STARTER_SCENE_FILE
            }))
            .expect("manifest json"),
        )
        .expect("manifest");

        let loaded =
            load_project_snapshot(root.display().to_string()).expect("legacy project loads");
        assert_eq!(loaded.document.name, "Login Button");
        assert!(loaded.operation_batches.is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_manifest_rejects_absolute_main_scene_paths() {
        let root = temp_project_root("absolute-main-scene");
        fs::create_dir_all(&root).expect("root dir");
        let absolute_scene = root.join("outside.strut");
        write_project_manifest(&root, "Bad Manifest", &absolute_scene.display().to_string());

        let error =
            load_project_snapshot(root.display().to_string()).expect_err("absolute path rejects");
        assert!(error.contains("mainScene path must be relative"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_manifest_rejects_traversal_main_scene_paths() {
        let root = temp_project_root("traversal-main-scene");
        fs::create_dir_all(&root).expect("root dir");
        write_project_manifest(&root, "Bad Manifest", "../outside.strut");

        let error =
            load_project_snapshot(root.display().to_string()).expect_err("traversal path rejects");
        assert!(error.contains("mainScene path must stay inside"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_manifest_accepts_valid_relative_main_scene_paths() {
        let root = temp_project_root("relative-main-scene");
        let document = strut_core::Document::sample_login_button();
        write_project_scene(&root, "scenes/custom.strut", &document);
        write_project_manifest(&root, "Custom Scene", "scenes/custom.strut");

        let loaded =
            load_project_snapshot(root.display().to_string()).expect("relative scene loads");
        assert_eq!(loaded.document.name, "Login Button");
        assert_eq!(loaded.main_scene, "scenes/custom.strut");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_manifest_scene_still_falls_back_to_legacy_scene() {
        let root = temp_project_root("missing-main-scene-fallback");
        fs::create_dir_all(root.join("scenes")).expect("scenes dir");
        let document = strut_core::Document::sample_login_button();
        fs::write(
            root.join(LEGACY_STARTER_SCENE_FILE),
            serde_json::to_string_pretty(&document).expect("document json"),
        )
        .expect("legacy scene");
        write_project_manifest(&root, "Legacy Fallback", "scenes/missing.strut");

        let loaded =
            load_project_snapshot(root.display().to_string()).expect("legacy fallback loads");
        assert_eq!(loaded.document.name, "Login Button");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sprite_python_batch_persists_only_after_rust_validation() {
        let validated = validate_generation_plan_batch(
            sprite_python_fixture("dice").to_string(),
            "sprite-python".to_string(),
            Some("rolling dice".to_string()),
        )
        .expect("sprite-python fixture validates");

        assert_eq!(validated.batch.source_type, "sprite-python");
        assert!(validated.batch.validation_result.ok);
        assert_eq!(validated.batch.status, "applied");
        assert!(validated.operation_count >= 10);
        assert!(validated
            .batch
            .operations
            .iter()
            .any(|operation| operation.get("type").and_then(Value::as_str) == Some("create_node")));

        let bad = validate_generation_plan_batch(
            json!({
                "plan": {
                    "id": "bad-logo",
                    "name": "Bad Logo",
                    "subject": {"classification": "logo", "label": "Logo"},
                    "parts": [
                        phase3_part("Body", "Body", "body", json!({"kind":"ellipse","cx":480,"cy":270,"rx":80,"ry":80})),
                        phase3_part("Head", "Head", "head", json!({"kind":"ellipse","cx":480,"cy":190,"rx":60,"ry":50})),
                        phase3_part("Eyes", "Eyes", "eyes", json!({"kind":"path","d":"M460 190 L470 190"})),
                        phase3_part("PrimaryMark", "PrimaryMark", "mark", json!({"kind":"path","d":"M420 240 L540 240"})),
                        phase3_part("AccentStroke", "AccentStroke", "accent", json!({"kind":"path","d":"M420 270 L540 270"}))
                    ],
                    "motionRoles": [{"id": "primary", "purpose": "bad mascot anatomy", "partRefs": ["PrimaryMark"]}],
                    "states": ["idle", "reveal"],
                    "timelines": [{
                        "id": "reveal",
                        "name": "reveal",
                        "state": "reveal",
                        "durationMs": 1000,
                        "tracks": [{
                            "target": "PrimaryMark",
                            "property": "opacity",
                            "keyframes": [
                                {"timeMs": 0, "value": 0, "easing": "linear"},
                                {"timeMs": 1000, "value": 1, "easing": "linear"}
                            ]
                        }]
                    }],
                    "editability": {"editableParts": ["PrimaryMark"], "lockedParts": [], "notes": []}
                },
                "operations": []
            })
            .to_string(),
            "sprite-python".to_string(),
            None,
        )
        .expect_err("invalid sprite-python batch rejects");
        assert!(bad.contains("mascot-only anatomy"));
    }

    #[test]
    fn local_agent_catalog_includes_requested_providers() {
        let adapters = local_agent_adapters();
        let ids = adapters
            .iter()
            .map(|adapter| adapter.id.as_str())
            .collect::<Vec<_>>();

        assert!(ids.contains(&"codex"));
        assert!(ids.contains(&"gemini-cli"));
        assert!(ids.contains(&"claude-code"));
        assert!(ids.contains(&"copilot-cli"));
        assert!(ids.contains(&"ollama"));
        assert!(ids.contains(&"opencode"));
        assert!(ids.contains(&"cursor-agent"));
        assert!(ids.contains(&"qwen"));
        assert!(ids.contains(&"qoder"));
    }

    #[test]
    fn parses_full_document_from_model_text() {
        let document_json = serde_json::to_string(&strut_core::Document::sample_owl_mascot())
            .expect("document json");
        let document =
            parse_generated_document(&format!("Here is JSON: {{\"document\":{document_json}}}"))
                .expect("document should parse");

        assert_eq!(document.name, "Owl Mascot");
        assert_eq!(document.artboards[0].name, "OwlMascot");
    }

    #[test]
    fn rejects_legacy_preset_spec_from_model_text() {
        let error = parse_generated_document(
            r##"{"variant":"owl-guide","name":"Owl Mascot","accent":"#78d64b","shell":"#8ee15a"}"##,
        )
        .expect_err("legacy spec should be rejected");

        assert!(error.contains("old preset spec format"));
    }

    #[test]
    fn normalizes_model_friendly_ids_and_partial_styles() {
        let document = parse_generated_document(
            r##"{
              "document": {
                "id": "doc",
                "name": "Loose Mascot",
                "artboards": [{
                  "id": "main-board",
                  "name": "Loose",
                  "width": 960,
                  "height": 540,
                  "nodes": [{
                    "id": "rig",
                    "name": "Rig",
                    "kind": "group",
                    "children": [
                      {"id":"body","name":"Body","kind":"ellipse","style":{"fill":"#78c137"},"shape":{"type":"ellipse","cx":480,"cy":270,"rx":100,"ry":120}},
                      {"id":"face","name":"Face","kind":"rect","style":{"fill":"none","stroke":"#111"},"shape":{"type":"rect","x":400,"y":220,"width":160,"height":90,"rx":24}},
                      {"id":"eye-a","name":"EyeA","kind":"ellipse","shape":{"type":"ellipse","cx":440,"cy":250,"rx":10,"ry":10}},
                      {"id":"eye-b","name":"EyeB","kind":"ellipse","shape":{"type":"ellipse","cx":520,"cy":250,"rx":10,"ry":10}},
                      {"id":"smile","name":"Smile","kind":"path","shape":{"type":"path","d":"M450 290 C480 315 510 290"}}
                    ]
                  }]
                }],
                "timelines": [{"id":"wave-line","name":"wave","duration":800,"tracks":[{"target":"rig","property":"translate_y","keyframes":[{"time":0,"value":0,"easing":"easeOutQuad"},{"time":400,"value":8,"easing":"easeInOutQuad"}]}]}],
                "state_machines": [{"id":"moods","name":"Moods","states":[{"id":"idle","name":"Idle"},{"id":"wave","name":"Wave"}],"transitions":[]}],
                "bindings": [],
                "events": []
              }
            }"##,
        )
        .expect("loose document should normalize");

        assert_eq!(document.name, "Loose Mascot");
        assert_eq!(
            document.timelines[0].tracks[0].target,
            document.artboards[0].nodes[0].id
        );
        assert_eq!(document.timelines[0].duration_ms, 800);
        assert_eq!(document.timelines[0].tracks[0].property, "translation.y");
        assert_eq!(document.timelines[0].tracks[0].keyframes[0].time_ms, 0);
        assert_eq!(
            document.artboards[0].nodes[0].children[0].style.opacity,
            1.0
        );
        assert_eq!(document.artboards[0].nodes[0].children[1].style.fill, None);
        assert!(document.state_machines[0]
            .states
            .contains(&"wave".to_string()));
    }

    #[test]
    fn parses_full_document_from_streaming_cli_output() {
        let document_text =
            serde_json::json!({"document": strut_core::Document::sample_minimal_bot()}).to_string();
        let text = cli_assistant_text(
            &serde_json::json!({"type":"assistant","message":{"content":document_text}})
                .to_string(),
        );
        let document = parse_generated_document(&text).expect("document should parse");

        assert_eq!(document.name, "Minimal Bot");
    }

    #[test]
    fn parses_full_document_from_gemini_delta_chunks() {
        let document_text =
            serde_json::json!({"document": strut_core::Document::sample_owl_mascot()}).to_string();
        let split_at = document_text.len() / 2;
        let (first, second) = document_text.split_at(split_at);
        let text = cli_assistant_text(&format!(
            "{}\n{}\n{}",
            serde_json::json!({"type":"message","role":"user","content":"Return a full Strut document."}),
            serde_json::json!({"type":"message","role":"assistant","content":first,"delta":true}),
            serde_json::json!({"type":"message","role":"assistant","content":second,"delta":true})
        ));
        let document = parse_generated_document(&text).expect("document should parse");

        assert_eq!(document.state_machines[0].name, "OwlMoods");
    }

    #[test]
    fn contextual_prompt_carries_chat_history_and_current_document() {
        let context = GenerationContext {
            project_name: Some("Mascot Game".to_string()),
            project_path: Some("D:\\Strut Projects\\Mascot Game".to_string()),
            active_chat_title: Some("Follow-up edits".to_string()),
            current_document_summary: Some(
                "Owl Mascot; 12 editable layers; states: idle, wave".to_string(),
            ),
            chat_history: vec![
                GenerationContextMessage {
                    role: "user".to_string(),
                    text: "make a green owl mascot".to_string(),
                    attachments: Some(vec!["owl-reference.png".to_string()]),
                },
                GenerationContextMessage {
                    role: "assistant".to_string(),
                    text: "Owl Mascot is ready.".to_string(),
                    attachments: None,
                },
            ],
            current_document: Some(strut_core::Document::sample_owl_mascot()),
        };

        let prompt =
            contextual_generation_prompt("make it cheer when level completes", Some(&context))
                .expect("context prompt");

        assert!(prompt.contains("Project: Mascot Game"));
        assert!(prompt.contains("make a green owl mascot"));
        assert!(prompt.contains("owl-reference.png"));
        assert!(prompt.contains("Current editable Strut document"));
        assert!(prompt.contains("Owl Mascot"));
        assert!(prompt.contains("make it cheer when level completes"));
    }

    #[test]
    fn repair_prompt_preserves_request_and_validation_error() {
        let prompt = document_repair_prompt(
            "make pikachu style character giving an electric shock",
            "I made a yellow mascot but forgot the JSON.",
            "model did not return a valid Strut document",
        );

        assert!(prompt.contains("make pikachu style character"));
        assert!(prompt.contains("model did not return a valid Strut document"));
        assert!(prompt.contains("Previous invalid response"));
        assert!(prompt.contains("{\"document\": <StrutDocument>}"));
        assert!(prompt.contains("Do not explain"));
    }

    #[test]
    fn compact_plan_compiles_to_valid_document() {
        let document = document_from_compact_plan_text(
            r##"{
              "plan": {
                "name": "Pika Shock",
                "motion": "electric cheer",
                "parts": [
                  {"name":"Yellow body","kind":"ellipse","x":480,"y":292,"width":170,"height":190,"fill":"#ffd84d","stroke":"#241f1a"},
                  {"name":"Left ear","kind":"path","d":"M420 190 L388 104 L458 168 Z","fill":"#ffd84d","stroke":"#241f1a"},
                  {"name":"Right ear","kind":"path","d":"M540 190 L578 104 L508 168 Z","fill":"#ffd84d","stroke":"#241f1a"},
                  {"name":"Face","kind":"rect","x":418,"y":238,"width":124,"height":72,"fill":"#fff6c7","stroke":"#241f1a"},
                  {"name":"Eyes","kind":"path","d":"M446 266 q12 -18 24 0 M494 266 q12 -18 24 0","fill":"none","stroke":"#241f1a"},
                  {"name":"Smile","kind":"path","d":"M456 292 Q480 318 510 292","fill":"none","stroke":"#241f1a"},
                  {"name":"Spark","kind":"path","d":"M600 236 L632 236 L610 272 L640 272 L590 330 L606 288 L578 288 Z","fill":"#ffe74a","stroke":"#241f1a"}
                ]
              }
            }"##,
        )
        .expect("compact plan should compile");

        assert_eq!(document.name, "Pika Shock");
        assert!(count_document_nodes(&document) >= 6);
        assert!(document.state_machines[0]
            .states
            .contains(&"celebrate".to_string()));
    }

    #[test]
    fn rolling_dice_plan_does_not_produce_mascot_anatomy() {
        let planned = document_from_generation_plan_text(&phase3_plan_text(
            "dice",
            "Rolling Dice",
            vec![
                phase3_part("DieBody", "DieBody", "volume", json!({"kind":"rect","x":378,"y":174,"width":210,"height":210,"rx":24})),
                phase3_part("FrontFace", "FrontFace", "front face", json!({"kind":"rect","x":402,"y":214,"width":168,"height":146,"rx":16})),
                phase3_part("TopFace", "TopFace", "top face", json!({"kind":"path","d":"M402 214 L454 168 L618 184 L570 214 Z"})),
                phase3_part("Pips", "Pips", "number marks", json!({"kind":"path","d":"M442 252 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0 M530 320 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0"})),
                phase3_part("EdgeHighlight", "EdgeHighlight", "edge light", json!({"kind":"path","d":"M414 228 L454 188 L604 202"})),
                phase3_part("SettleShadow", "SettleShadow", "grounding shadow", json!({"kind":"ellipse","cx":494,"cy":414,"rx":116,"ry":18})),
            ],
            "settle",
            "DieBody",
        ))
        .expect("dice plan should convert");

        let names = semantic_layer_names(&planned.document);
        assert!(names.iter().any(|name| name == "DieBody"));
        assert!(names.iter().any(|name| name == "Pips"));
        assert!(names
            .iter()
            .all(|name| !matches!(name.as_str(), "Head" | "Eyes" | "Arms" | "Legs" | "Smile")));
        assert!(planned
            .summary
            .timeline_names
            .contains(&"settle".to_string()));
        assert!(planned.operation_count >= 10);
    }

    #[test]
    fn abstract_logo_plan_does_not_require_face() {
        let planned = document_from_generation_plan_text(&phase3_plan_text(
            "logo",
            "Abstract Logo",
            vec![
                phase3_part("PrimaryMark", "PrimaryMark", "main vector mark", json!({"kind":"path","d":"M382 180 C450 120 540 146 582 222 C520 206 470 234 432 306 C398 266 370 226 382 180 Z"})),
                phase3_part("Wordmark", "Wordmark", "brand text", json!({"kind":"text","x":396,"y":384,"value":"STRUT","size":42})),
                phase3_part("AccentStroke", "AccentStroke", "accent line", json!({"kind":"path","d":"M392 326 C452 352 528 348 596 312"})),
                phase3_part("RevealMask", "RevealMask", "reveal mask", json!({"kind":"rect","x":360,"y":154,"width":280,"height":250,"rx":20})),
                phase3_part("AnchorGrid", "AnchorGrid", "alignment grid", json!({"kind":"path","d":"M360 270 L640 270 M500 150 L500 410"})),
                phase3_part("Glow", "Glow", "soft emphasis", json!({"kind":"ellipse","cx":498,"cy":266,"rx":118,"ry":76})),
            ],
            "reveal",
            "PrimaryMark",
        ))
        .expect("logo plan should convert");

        let names = semantic_layer_names(&planned.document);
        assert!(names.iter().any(|name| name == "PrimaryMark"));
        assert!(names.iter().any(|name| name == "Wordmark"));
        assert!(names.iter().all(|name| name != "Face" && name != "Eyes"));
    }

    #[test]
    fn loader_plan_does_not_require_face_or_body() {
        let planned = document_from_generation_plan_text(&phase3_plan_text(
            "loader",
            "Progress Loader",
            vec![
                phase3_part(
                    "Track",
                    "Track",
                    "background track",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":120,"ry":120}),
                ),
                phase3_part(
                    "ActiveSegment",
                    "ActiveSegment",
                    "active arc",
                    json!({"kind":"path","d":"M480 150 A120 120 0 0 1 600 270"}),
                ),
                phase3_part(
                    "PulseDot",
                    "PulseDot",
                    "pulse marker",
                    json!({"kind":"ellipse","cx":600,"cy":270,"rx":14,"ry":14}),
                ),
                phase3_part(
                    "ProgressSweep",
                    "ProgressSweep",
                    "sweep indicator",
                    json!({"kind":"path","d":"M480 270 L600 270"}),
                ),
                phase3_part(
                    "Glow",
                    "Glow",
                    "soft glow",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":144,"ry":144}),
                ),
                phase3_part(
                    "CenterLabel",
                    "CenterLabel",
                    "progress label",
                    json!({"kind":"text","x":454,"y":282,"value":"42%","size":24}),
                ),
            ],
            "loading",
            "ActiveSegment",
        ))
        .expect("loader plan should convert");

        let names = semantic_layer_names(&planned.document);
        assert!(names.iter().any(|name| name == "ActiveSegment"));
        assert!(names.iter().all(|name| name != "Face" && name != "Body"));
        assert!(planned.document.state_machines[0]
            .states
            .contains(&"loading".to_string()));
    }

    #[test]
    fn mascot_plan_can_still_use_mascot_parts() {
        let planned = document_from_generation_plan_text(&phase3_plan_text(
            "mascot",
            "Helpful Mascot",
            vec![
                phase3_part("Body", "Body", "body", json!({"kind":"ellipse","cx":480,"cy":306,"rx":92,"ry":118})),
                phase3_part("Head", "Head", "head", json!({"kind":"ellipse","cx":480,"cy":190,"rx":82,"ry":68})),
                phase3_part("Eyes", "Eyes", "eyes", json!({"kind":"path","d":"M446 186 q10 -16 20 0 M494 186 q10 -16 20 0"})),
                phase3_part("Arms", "Arms", "arms", json!({"kind":"path","d":"M394 292 C350 310 344 352 382 364 M566 292 C610 310 616 352 578 364"})),
                phase3_part("AccentBadge", "AccentBadge", "accent", json!({"kind":"ellipse","cx":512,"cy":316,"rx":16,"ry":16})),
                phase3_part("GroundShadow", "GroundShadow", "shadow", json!({"kind":"ellipse","cx":480,"cy":438,"rx":108,"ry":16})),
            ],
            "wave",
            "Body",
        ))
        .expect("mascot plan should convert");

        let names = semantic_layer_names(&planned.document);
        assert!(names.iter().any(|name| name == "Body"));
        assert!(names.iter().any(|name| name == "Head"));
        assert!(names.iter().any(|name| name == "Eyes"));
    }

    #[test]
    fn generation_plans_reject_invalid_references_and_geometry() {
        let duplicate = phase3_plan_text(
            "logo",
            "Bad Logo",
            vec![
                phase3_part(
                    "PrimaryMark",
                    "PrimaryMark",
                    "main",
                    json!({"kind":"path","d":"M0 0 L10 10"}),
                ),
                phase3_part(
                    "PrimaryMark",
                    "AccentStroke",
                    "accent",
                    json!({"kind":"path","d":"M0 10 L10 0"}),
                ),
                phase3_part(
                    "RevealMask",
                    "RevealMask",
                    "mask",
                    json!({"kind":"rect","x":1,"y":1,"width":10,"height":10,"rx":2}),
                ),
                phase3_part(
                    "AnchorGrid",
                    "AnchorGrid",
                    "grid",
                    json!({"kind":"path","d":"M1 1 L10 1"}),
                ),
                phase3_part(
                    "Glow",
                    "Glow",
                    "glow",
                    json!({"kind":"ellipse","cx":5,"cy":5,"rx":4,"ry":4}),
                ),
            ],
            "reveal",
            "PrimaryMark",
        );
        assert!(document_from_generation_plan_text(&duplicate)
            .expect_err("duplicate ids should reject")
            .contains("duplicate part id"));

        let missing_target = phase3_plan_text(
            "loader",
            "Bad Loader",
            vec![
                phase3_part(
                    "Track",
                    "Track",
                    "track",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":120,"ry":120}),
                ),
                phase3_part(
                    "ActiveSegment",
                    "ActiveSegment",
                    "active",
                    json!({"kind":"path","d":"M480 150 A120 120 0 0 1 600 270"}),
                ),
                phase3_part(
                    "PulseDot",
                    "PulseDot",
                    "dot",
                    json!({"kind":"ellipse","cx":600,"cy":270,"rx":14,"ry":14}),
                ),
                phase3_part(
                    "ProgressSweep",
                    "ProgressSweep",
                    "sweep",
                    json!({"kind":"path","d":"M480 270 L600 270"}),
                ),
                phase3_part(
                    "Glow",
                    "Glow",
                    "glow",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":144,"ry":144}),
                ),
            ],
            "loading",
            "MissingPart",
        );
        assert!(document_from_generation_plan_text(&missing_target)
            .expect_err("unknown timeline target should reject")
            .contains("missing part"));

        let bad_geometry = phase3_plan_text(
            "dice",
            "Bad Dice",
            vec![
                phase3_part(
                    "DieBody",
                    "DieBody",
                    "body",
                    json!({"kind":"rect","x":1,"y":1,"width":0,"height":10,"rx":2}),
                ),
                phase3_part(
                    "FrontFace",
                    "FrontFace",
                    "face",
                    json!({"kind":"rect","x":1,"y":1,"width":10,"height":10,"rx":2}),
                ),
                phase3_part(
                    "TopFace",
                    "TopFace",
                    "face",
                    json!({"kind":"path","d":"M0 0 L10 10"}),
                ),
                phase3_part(
                    "Pips",
                    "Pips",
                    "pips",
                    json!({"kind":"path","d":"M1 1 L2 2"}),
                ),
                phase3_part(
                    "Shadow",
                    "Shadow",
                    "shadow",
                    json!({"kind":"ellipse","cx":5,"cy":5,"rx":4,"ry":4}),
                ),
            ],
            "settle",
            "DieBody",
        );
        assert!(document_from_generation_plan_text(&bad_geometry)
            .expect_err("invalid geometry should reject")
            .contains("invalid rect geometry"));
    }

    #[test]
    fn non_mascot_plan_rejects_mascot_only_anatomy() {
        let bad_logo = phase3_plan_text(
            "logo",
            "Logo With Face",
            vec![
                phase3_part(
                    "Body",
                    "Body",
                    "body",
                    json!({"kind":"ellipse","cx":480,"cy":270,"rx":80,"ry":80}),
                ),
                phase3_part(
                    "Head",
                    "Head",
                    "head",
                    json!({"kind":"ellipse","cx":480,"cy":190,"rx":60,"ry":50}),
                ),
                phase3_part(
                    "Eyes",
                    "Eyes",
                    "eyes",
                    json!({"kind":"path","d":"M460 190 L470 190"}),
                ),
                phase3_part(
                    "PrimaryMark",
                    "PrimaryMark",
                    "mark",
                    json!({"kind":"path","d":"M420 240 L540 240"}),
                ),
                phase3_part(
                    "AccentStroke",
                    "AccentStroke",
                    "accent",
                    json!({"kind":"path","d":"M420 270 L540 270"}),
                ),
            ],
            "reveal",
            "PrimaryMark",
        );

        assert!(document_from_generation_plan_text(&bad_logo)
            .expect_err("non mascot anatomy should reject")
            .contains("mascot-only anatomy"));
    }

    #[test]
    fn open_project_folder_rejects_missing_folder() {
        let missing =
            std::env::temp_dir().join(format!("strut-missing-folder-{}", unix_timestamp()));

        let error = open_project_folder(missing.display().to_string())
            .expect_err("missing folders should not be opened");

        assert!(error.contains("Project folder does not exist"));
    }

    #[test]
    fn provider_config_path_is_local() {
        let path = provider_config_path().expect("config path");
        assert!(path.ends_with("byok.json"));
    }

    #[test]
    fn endpoint_guard_allows_loopback_and_blocks_private_networks() {
        assert!(ensure_safe_endpoint("http://localhost:1234/v1").is_ok());
        assert!(ensure_safe_endpoint("http://127.0.0.1:11434").is_ok());
        assert!(ensure_safe_endpoint("https://api.openai.com/v1").is_ok());
        assert!(ensure_safe_endpoint("http://192.168.1.20:8080").is_err());
        assert!(ensure_safe_endpoint("ftp://api.example.com").is_err());
    }

    #[test]
    fn windows_command_candidates_prefer_executable_shims() {
        let candidates = command_candidates("gemini");
        if cfg!(windows) {
            let first = candidates
                .first()
                .and_then(|path| path.extension())
                .and_then(|extension| extension.to_str())
                .unwrap_or_default()
                .to_lowercase();
            assert_ne!(first, "");
        } else {
            assert_eq!(candidates, vec![PathBuf::from("gemini")]);
        }
    }

    #[test]
    #[ignore = "requires authenticated Gemini CLI"]
    fn gemini_cli_generates_owl_mascot_end_to_end() {
        let document = tauri::async_runtime::block_on(generate_document_with_local_adapter(
            "gemini-cli",
            "Make a friendly owl style mascot like Duo. Keep it simple and editable, with wave, blink, scan, and celebrate animation states.",
            &[],
        ))
        .expect("Gemini CLI should return a full Strut document")
        .document;
        let mut layer_names = Vec::new();
        collect_layer_names(&document.artboards[0].nodes, &mut layer_names);
        let states = &document.state_machines[0].states;

        assert!(document.name.to_lowercase().contains("owl"));
        assert!(document.artboards[0].nodes.len() >= 3 || layer_names.len() >= 6);
        assert!(layer_names
            .iter()
            .any(|name| name.to_lowercase().contains("owl")));
        for state in ["wave", "blink", "scan", "celebrate"] {
            assert!(states.iter().any(|item| item == state));
        }
    }

    #[test]
    fn project_name_is_sanitized() {
        assert_eq!(
            sanitize_project_name("  My Bot / Demo!! ").expect("project name"),
            "My Bot Demo"
        );
    }
}
