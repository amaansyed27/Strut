use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use strut_core::Document;
use thiserror::Error;

const PROJECT_MANIFEST_FILE: &str = "strut.project.json";
const MAIN_SCENE_FILE: &str = "scenes/main.strut";
const OPERATION_BATCHES_FILE: &str = "operations/operation-batches.json";
const STUDIO_STATE_FILE: &str = "ui/studio-state.json";

#[derive(Debug, Parser)]
#[command(name = "strut", about = "Agentic CLI for validated Strut scenes")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Inspect(InspectCommand),
    Plan(PlanArgs),
    Sprite(SpriteCommand),
    Patch(PatchArgs),
    Verify(VerifyArgs),
    Render(RenderArgs),
    Export(ExportCommand),
}

#[derive(Debug, Args)]
struct InspectCommand {
    #[command(subcommand)]
    target: InspectTarget,
}

#[derive(Debug, Subcommand)]
enum InspectTarget {
    Project(InspectProjectArgs),
    Scene(InspectSceneArgs),
}

#[derive(Debug, Args)]
struct InspectProjectArgs {
    #[arg(default_value = ".")]
    project_dir: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct InspectSceneArgs {
    scene_file: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct PlanArgs {
    instruction: String,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    explain: bool,
}

#[derive(Debug, Args)]
struct SpriteCommand {
    #[command(subcommand)]
    command: SpriteSubcommand,
}

#[derive(Debug, Subcommand)]
enum SpriteSubcommand {
    Plan(PlanArgs),
}

#[derive(Debug, Args)]
struct PatchArgs {
    #[arg(long)]
    scene: PathBuf,
    #[arg(long = "from")]
    from: PathBuf,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct VerifyArgs {
    scene_file: PathBuf,
    #[arg(long = "batch")]
    batch_files: Vec<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct RenderArgs {
    #[arg(long)]
    scene: PathBuf,
    #[arg(long)]
    state: String,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    no_open: bool,
}

#[derive(Debug, Args)]
struct ExportCommand {
    #[command(subcommand)]
    target: ExportTarget,
}

#[derive(Debug, Subcommand)]
enum ExportTarget {
    React(ExportReactArgs),
}

#[derive(Debug, Args)]
struct ExportReactArgs {
    #[arg(long)]
    scene: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Error)]
enum CliError {
    #[error("{0}")]
    Message(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("format error: {0}")]
    Format(#[from] strut_format::FormatError),
}

type CliResult<T> = Result<T, CliError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SceneInspection {
    scene_file: String,
    validation: ValidationStatus,
    summary: DocumentSummary,
    artboards: Vec<ArtboardSummary>,
    nodes: Vec<NodeSummary>,
    timelines: Vec<TimelineSummary>,
    states: Vec<StateMachineSummary>,
    events: Vec<String>,
    semantic_roles: Vec<SemanticRoleSummary>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectInspection {
    project_dir: String,
    canonical_files: Vec<ProjectFileSummary>,
    main_scene: Option<String>,
    operation_batches: Vec<BatchSummary>,
    current_document: Option<DocumentSummary>,
    timelines: Vec<TimelineSummary>,
    states: Vec<StateMachineSummary>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFileSummary {
    role: String,
    path: String,
    exists: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ValidationStatus {
    ok: bool,
    message: String,
    validator: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DocumentSummary {
    id: String,
    name: String,
    artboards: usize,
    nodes: usize,
    timelines: usize,
    state_machines: usize,
    bindings: usize,
    events: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtboardSummary {
    id: String,
    name: String,
    width: f32,
    height: f32,
    node_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeSummary {
    id: String,
    name: String,
    kind: String,
    role: Option<String>,
    children: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TimelineSummary {
    id: String,
    name: String,
    duration_ms: u32,
    tracks: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StateMachineSummary {
    id: String,
    name: String,
    states: Vec<String>,
    transitions: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticRoleSummary {
    node_id: String,
    name: String,
    role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct OperationValidationResult {
    ok: bool,
    message: String,
    validator: String,
    validated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchSummary {
    id: String,
    source_type: String,
    status: String,
    operation_count: usize,
    validation_ok: bool,
    document_revision_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CliPlanFile {
    format: String,
    instruction: String,
    backend: String,
    dry_run: bool,
    explanation: Option<String>,
    plan_summary: GenerationPlanSummary,
    envelope: Value,
    document: Document,
    batch: OperationBatchRecord,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GenerationPlanSummary {
    subject_classification: String,
    subject_label: String,
    part_names: Vec<String>,
    timeline_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerationPlanEnvelope {
    plan: GenerationPlan,
    #[serde(default)]
    operations: Vec<SceneOperation>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubjectPlan {
    classification: String,
    label: String,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanStyle {
    fill: Option<String>,
    stroke: Option<String>,
    #[serde(default, alias = "stroke_width")]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditabilityPlan {
    #[serde(default)]
    editable_parts: Vec<String>,
    #[serde(default)]
    locked_parts: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MotionRolePlan {
    id: String,
    purpose: String,
    #[serde(default)]
    part_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimelinePlan {
    id: String,
    name: String,
    state: Option<String>,
    #[serde(alias = "duration_ms")]
    duration_ms: u32,
    #[serde(default)]
    tracks: Vec<TimelineTrackPlan>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimelineTrackPlan {
    target: String,
    property: String,
    #[serde(default)]
    keyframes: Vec<KeyframePlan>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeyframePlan {
    #[serde(alias = "time_ms")]
    time_ms: u32,
    value: f64,
    easing: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug)]
struct PlannedDocument {
    document: Document,
    summary: GenerationPlanSummary,
    operations: Vec<SceneOperation>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("strut: {error}");
        std::process::exit(1);
    }
}

fn run() -> CliResult<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Inspect(command) => match command.target {
            InspectTarget::Project(args) => inspect_project(args),
            InspectTarget::Scene(args) => inspect_scene(args),
        },
        Commands::Plan(args) => plan_command(args, "local-fixture"),
        Commands::Sprite(command) => match command.command {
            SpriteSubcommand::Plan(args) => plan_command(args, "sprite-python"),
        },
        Commands::Patch(args) => patch_command(args),
        Commands::Verify(args) => verify_command(args),
        Commands::Render(args) => render_command(args),
        Commands::Export(command) => match command.target {
            ExportTarget::React(args) => export_react_command(args),
        },
    }
}

fn inspect_project(args: InspectProjectArgs) -> CliResult<()> {
    let root = args.project_dir;
    let manifest_path = root.join(PROJECT_MANIFEST_FILE);
    let main_scene = read_project_main_scene(&root)?;
    let scene_path = main_scene
        .as_ref()
        .map(|path| root.join(path))
        .unwrap_or_else(|| root.join(MAIN_SCENE_FILE));
    let operations_path = root.join(OPERATION_BATCHES_FILE);
    let state_path = root.join(STUDIO_STATE_FILE);

    let canonical_files = vec![
        project_file("manifest", &manifest_path),
        project_file("mainScene", &scene_path),
        project_file("operationBatches", &operations_path),
        project_file("studioState", &state_path),
    ];
    let mut warnings = Vec::new();
    let mut document = None;
    if scene_path.exists() {
        match read_document(&scene_path) {
            Ok(read) => document = Some(read),
            Err(error) => warnings.push(format!("main scene is invalid: {error}")),
        }
    } else {
        warnings.push(format!("main scene is missing: {}", scene_path.display()));
    }

    if !manifest_path.exists() {
        warnings.push("project manifest is missing".to_string());
    }

    let operation_batches = if operations_path.exists() {
        let raw = fs::read_to_string(&operations_path)?;
        serde_json::from_str::<Vec<OperationBatchRecord>>(&raw)
            .map(|batches| batches.iter().map(batch_summary).collect::<Vec<_>>())
            .map_err(|error| {
                CliError::Message(format!(
                    "operation batch file is not parseable JSON: {error}"
                ))
            })?
    } else {
        warnings.push("operation batch file is missing".to_string());
        Vec::new()
    };

    let (current_document, timelines, states) = if let Some(document) = &document {
        (
            Some(document_summary(document)),
            timeline_summaries(document),
            state_summaries(document),
        )
    } else {
        (None, Vec::new(), Vec::new())
    };

    let report = ProjectInspection {
        project_dir: path_string(&root),
        canonical_files,
        main_scene: main_scene.or_else(|| Some(MAIN_SCENE_FILE.to_string())),
        operation_batches,
        current_document,
        timelines,
        states,
        warnings,
    };
    output(args.json, &report, || human_project_report(&report))
}

fn inspect_scene(args: InspectSceneArgs) -> CliResult<()> {
    let report = scene_inspection(&args.scene_file)?;
    output(args.json, &report, || human_scene_report(&report))
}

fn plan_command(args: PlanArgs, backend: &str) -> CliResult<()> {
    let (envelope, actual_backend, warnings) = if backend == "sprite-python" {
        sprite_python_envelope(&args.instruction)?
    } else {
        (
            fixture_envelope(&instruction_kind(&args.instruction))?,
            "local-fixture".to_string(),
            Vec::new(),
        )
    };
    let planned = document_from_generation_plan_value(&envelope)?;
    let document_revision_id = document_revision_id(&planned.document);
    let timestamp = "deterministic-cli-validation".to_string();
    let operations = vec![json!({
        "id": "op-replace-document",
        "type": "replace_document",
        "previousDocument": null,
        "nextDocument": planned.document
    })];
    let batch = OperationBatchRecord {
        id: format!("batch-cli-{document_revision_id}"),
        source_type: if backend == "sprite-python" {
            "sprite-python".to_string()
        } else {
            "cli".to_string()
        },
        status: "pending".to_string(),
        validation_result: OperationValidationResult {
            ok: true,
            message: "Rust validated operation plan; patch will revalidate before writing"
                .to_string(),
            validator: "strut-cli-rust".to_string(),
            validated_at: timestamp.clone(),
        },
        document_revision_id,
        previous_document_revision_id: None,
        prompt: Some(args.instruction.clone()),
        source_metadata: Some(json!({
            "backend": actual_backend,
            "rawOperationCount": planned.operations.len(),
            "subjectClassification": planned.summary.subject_classification,
            "subjectLabel": planned.summary.subject_label
        })),
        operations,
        created_at: timestamp.clone(),
        updated_at: timestamp,
        applied_at: None,
        rejected_at: None,
    };
    validate_operation_batch(&batch)?;
    let explanation = args.explain.then(|| {
        "Instruction resolved to a deterministic generation-plan envelope, converted to a Strut document, and wrapped in a pending replace_document batch. No files are mutated by plan commands.".to_string()
    });
    let plan = CliPlanFile {
        format: "strut.cli.plan.v1".to_string(),
        instruction: args.instruction,
        backend: actual_backend,
        dry_run: args.dry_run,
        explanation,
        plan_summary: planned.summary,
        envelope,
        document: planned.document,
        batch,
        warnings,
    };
    output(args.json, &plan, || human_plan_report(&plan))
}

fn patch_command(args: PatchArgs) -> CliResult<()> {
    let current = read_document(&args.scene)?;
    let raw = fs::read_to_string(&args.from)?;
    let mut plan: CliPlanFile = serde_json::from_str(trim_json_input(&raw)).map_err(|error| {
        CliError::Message(format!(
            "plan file '{}' is not a strut plan JSON object: {error}",
            args.from.display()
        ))
    })?;
    if plan.format != "strut.cli.plan.v1" {
        return Err(CliError::Message(format!(
            "unsupported plan format '{}'",
            plan.format
        )));
    }
    let next_document = authoritative_replacement_document(&mut plan, &current)?;

    let summary = json!({
        "ok": true,
        "dryRun": args.dry_run,
        "scene": path_string(&args.scene),
        "plan": path_string(&args.from),
        "previousDocument": document_summary(&current),
        "nextDocument": document_summary(&next_document),
        "batch": batch_summary(&plan.batch),
        "message": if args.dry_run { "validated patch without writing scene" } else { "validated patch and wrote scene" }
    });

    if !args.dry_run {
        write_document(&args.scene, &next_document)?;
    }
    output(args.json, &summary, || {
        if args.dry_run {
            format!(
                "Patch validated for {}. Dry run: no files changed.",
                args.scene.display()
            )
        } else {
            format!("Patch validated and wrote {}.", args.scene.display())
        }
    })
}

fn verify_command(args: VerifyArgs) -> CliResult<()> {
    let document = read_document(&args.scene_file)?;
    let mut batches = Vec::new();
    for batch_file in &args.batch_files {
        let raw = fs::read_to_string(batch_file)?;
        if let Ok(batch) = serde_json::from_str::<OperationBatchRecord>(trim_json_input(&raw)) {
            validate_operation_batch(&batch)?;
            batches.push(batch_summary(&batch));
        } else {
            let batch_list =
                serde_json::from_str::<Vec<OperationBatchRecord>>(trim_json_input(&raw))?;
            for batch in &batch_list {
                validate_operation_batch(batch)?;
            }
            batches.extend(batch_list.iter().map(batch_summary));
        }
    }
    let report = json!({
        "ok": true,
        "scene": path_string(&args.scene_file),
        "validation": {
            "ok": true,
            "message": "scene document is valid",
            "validator": "strut-format"
        },
        "summary": document_summary(&document),
        "operationBatches": batches
    });
    output(args.json, &report, || {
        format!(
            "valid: {}\ndocument: {}\nartboards: {}\ntimelines: {}\nstate machines: {}\noperation batch files: {}",
            args.scene_file.display(),
            document.name,
            document.artboards.len(),
            document.timelines.len(),
            document.state_machines.len(),
            args.batch_files.len()
        )
    })
}

fn render_command(args: RenderArgs) -> CliResult<()> {
    let document = read_document(&args.scene)?;
    let plan = strut_renderer::plan_render(&document, strut_renderer::RenderBackend::CpuFallback);
    let svg = render_svg_proof(&document, &args.state);
    if let Some(parent) = args.out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.out, svg)?;
    let report = json!({
        "ok": true,
        "scene": path_string(&args.scene),
        "state": args.state,
        "out": path_string(&args.out),
        "backend": "cpu-fallback-svg-proof",
        "rendererPlan": {
            "backend": format!("{:?}", plan.backend),
            "artboardCount": plan.artboard_count
        },
        "opened": false,
        "limitations": ["The current renderer crate exposes render planning only, so this command writes a deterministic SVG proof of the scene structure."]
    });
    output(args.json, &report, || {
        format!(
            "render proof written: {}\nbackend: cpu-fallback-svg-proof\nnote: full renderer is not available yet",
            args.out.display()
        )
    })
}

fn export_react_command(args: ExportReactArgs) -> CliResult<()> {
    let document = read_document(&args.scene)?;
    let files = react_export_files(&document);
    let planned = files
        .iter()
        .map(|(path, _)| path_string(&args.out.join(path)))
        .collect::<Vec<_>>();
    if !args.dry_run {
        let targets = files
            .iter()
            .map(|(relative, content)| (args.out.join(relative), content))
            .collect::<Vec<_>>();
        let conflicts = targets
            .iter()
            .filter(|(target, _)| target.exists())
            .map(|(target, _)| path_string(target))
            .collect::<Vec<_>>();
        if !args.force && !conflicts.is_empty() {
            return Err(CliError::Message(format!(
                "refusing to overwrite existing export file(s): {}; pass --force to replace them",
                conflicts.join(", ")
            )));
        }
        fs::create_dir_all(&args.out)?;
        for (target, content) in targets {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target, content)?;
        }
    }
    let report = json!({
        "ok": true,
        "dryRun": args.dry_run,
        "scene": path_string(&args.scene),
        "out": path_string(&args.out),
        "files": planned,
        "message": if args.dry_run { "validated export without writing files" } else { "wrote React integration files" }
    });
    output(args.json, &report, || {
        if args.dry_run {
            format!(
                "React export validated for {}. Dry run: no files changed.",
                args.out.display()
            )
        } else {
            format!("React export written to {}.", args.out.display())
        }
    })
}

fn scene_inspection(scene_file: &Path) -> CliResult<SceneInspection> {
    let document = read_document(scene_file)?;
    let validation = match strut_format::validate_document(&document) {
        Ok(()) => ValidationStatus {
            ok: true,
            message: "scene document is valid".to_string(),
            validator: "strut-format".to_string(),
        },
        Err(error) => ValidationStatus {
            ok: false,
            message: error.to_string(),
            validator: "strut-format".to_string(),
        },
    };
    let mut warnings = Vec::new();
    if document.timelines.is_empty() {
        warnings.push("scene has no timelines".to_string());
    }
    if document.state_machines.is_empty() {
        warnings.push("scene has no state machines".to_string());
    }
    Ok(SceneInspection {
        scene_file: path_string(scene_file),
        validation,
        summary: document_summary(&document),
        artboards: document.artboards.iter().map(artboard_summary).collect(),
        nodes: node_summaries(&document),
        timelines: timeline_summaries(&document),
        states: state_summaries(&document),
        events: document
            .events
            .iter()
            .map(|event| event.name.clone())
            .collect(),
        semantic_roles: semantic_role_summaries(&document),
        warnings,
    })
}

fn read_project_main_scene(root: &Path) -> CliResult<Option<String>> {
    let manifest_path = root.join(PROJECT_MANIFEST_FILE);
    if !manifest_path.exists() {
        return Ok(None);
    }
    let manifest: Value = serde_json::from_str(&fs::read_to_string(manifest_path)?)?;
    Ok(manifest
        .get("mainScene")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| Some(MAIN_SCENE_FILE.to_string())))
}

fn project_file(role: &str, path: &Path) -> ProjectFileSummary {
    ProjectFileSummary {
        role: role.to_string(),
        path: path_string(path),
        exists: path.exists(),
    }
}

fn read_document(path: &Path) -> CliResult<Document> {
    Ok(strut_format::read_strut_file(path)?.document)
}

fn write_document(path: &Path, document: &Document) -> CliResult<()> {
    strut_format::write_strut_file(path, &strut_format::StrutPackage::current(document.clone()))?;
    Ok(())
}

fn output<T: Serialize>(
    json_output: bool,
    value: &T,
    human: impl FnOnce() -> String,
) -> CliResult<()> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", human());
    }
    Ok(())
}

fn path_string(path: &Path) -> String {
    path.display().to_string()
}

fn trim_json_input(raw: &str) -> &str {
    raw.trim_start_matches('\u{feff}').trim_start()
}

fn authoritative_replacement_document(
    plan: &mut CliPlanFile,
    current: &Document,
) -> CliResult<Document> {
    strut_format::validate_document(&plan.document)?;
    if plan.batch.operations.len() != 1 {
        return Err(CliError::Message(format!(
            "patch requires exactly one authoritative replace_document operation; found {} operations",
            plan.batch.operations.len()
        )));
    }
    let operation = plan
        .batch
        .operations
        .get_mut(0)
        .ok_or_else(|| CliError::Message("patch plan is missing operations".to_string()))?;
    if operation.get("type").and_then(Value::as_str) != Some("replace_document") {
        return Err(CliError::Message(
            "patch requires the single operation to be replace_document".to_string(),
        ));
    }
    let operation_map = operation.as_object_mut().ok_or_else(|| {
        CliError::Message("replace_document operation must be a JSON object".to_string())
    })?;
    operation_map.insert(
        "previousDocument".to_string(),
        serde_json::to_value(current)?,
    );
    let next_document_value = operation_map.get("nextDocument").ok_or_else(|| {
        CliError::Message("replace_document operation needs nextDocument".to_string())
    })?;
    let next_document: Document =
        serde_json::from_value(next_document_value.clone()).map_err(|error| {
            CliError::Message(format!(
                "replace_document nextDocument is not a valid Strut document: {error}"
            ))
        })?;
    strut_format::validate_document(&next_document).map_err(|error| {
        CliError::Message(format!(
            "replace_document nextDocument failed validation: {error}"
        ))
    })?;
    if plan.document != next_document {
        return Err(CliError::Message(
            "plan document mismatch: top-level document must exactly match batch.operations[0].nextDocument".to_string(),
        ));
    }
    plan.batch.previous_document_revision_id = Some(document_revision_id(current));
    validate_operation_batch(&plan.batch)?;
    Ok(next_document)
}

fn document_summary(document: &Document) -> DocumentSummary {
    DocumentSummary {
        id: document.id.to_string(),
        name: document.name.clone(),
        artboards: document.artboards.len(),
        nodes: count_document_nodes(document),
        timelines: document.timelines.len(),
        state_machines: document.state_machines.len(),
        bindings: document.bindings.len(),
        events: document.events.len(),
    }
}

fn artboard_summary(artboard: &strut_core::Artboard) -> ArtboardSummary {
    ArtboardSummary {
        id: artboard.id.to_string(),
        name: artboard.name.clone(),
        width: artboard.width,
        height: artboard.height,
        node_count: count_nodes(&artboard.nodes),
    }
}

fn node_summaries(document: &Document) -> Vec<NodeSummary> {
    let mut nodes = Vec::new();
    for artboard in &document.artboards {
        collect_node_summaries(&artboard.nodes, &mut nodes);
    }
    nodes
}

fn collect_node_summaries(nodes: &[strut_core::Node], summaries: &mut Vec<NodeSummary>) {
    for node in nodes {
        summaries.push(NodeSummary {
            id: node.id.to_string(),
            name: node.name.clone(),
            kind: format!("{:?}", node.kind).to_lowercase(),
            role: node.role.clone(),
            children: node.children.len(),
        });
        collect_node_summaries(&node.children, summaries);
    }
}

fn timeline_summaries(document: &Document) -> Vec<TimelineSummary> {
    document
        .timelines
        .iter()
        .map(|timeline| TimelineSummary {
            id: timeline.id.to_string(),
            name: timeline.name.clone(),
            duration_ms: timeline.duration_ms,
            tracks: timeline.tracks.len(),
        })
        .collect()
}

fn state_summaries(document: &Document) -> Vec<StateMachineSummary> {
    document
        .state_machines
        .iter()
        .map(|machine| StateMachineSummary {
            id: machine.id.to_string(),
            name: machine.name.clone(),
            states: machine.states.clone(),
            transitions: machine.transitions.len(),
        })
        .collect()
}

fn semantic_role_summaries(document: &Document) -> Vec<SemanticRoleSummary> {
    let mut roles = Vec::new();
    for artboard in &document.artboards {
        collect_roles(&artboard.nodes, &mut roles);
    }
    roles
}

fn collect_roles(nodes: &[strut_core::Node], roles: &mut Vec<SemanticRoleSummary>) {
    for node in nodes {
        if let Some(role) = &node.role {
            roles.push(SemanticRoleSummary {
                node_id: node.id.to_string(),
                name: node.name.clone(),
                role: role.clone(),
            });
        }
        collect_roles(&node.children, roles);
    }
}

fn count_document_nodes(document: &Document) -> usize {
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

fn batch_summary(batch: &OperationBatchRecord) -> BatchSummary {
    BatchSummary {
        id: batch.id.clone(),
        source_type: batch.source_type.clone(),
        status: batch.status.clone(),
        operation_count: batch.operations.len(),
        validation_ok: batch.validation_result.ok,
        document_revision_id: batch.document_revision_id.clone(),
    }
}

fn human_project_report(report: &ProjectInspection) -> String {
    let document = report
        .current_document
        .as_ref()
        .map(|summary| format!("{} ({} nodes)", summary.name, summary.nodes))
        .unwrap_or_else(|| "none".to_string());
    format!(
        "project: {}\nmain scene: {}\ndocument: {}\noperation batches: {}\ntimelines: {}\nstates: {}\nwarnings: {}",
        report.project_dir,
        report.main_scene.as_deref().unwrap_or("unknown"),
        document,
        report.operation_batches.len(),
        report.timelines.len(),
        report.states.iter().map(|state| state.states.len()).sum::<usize>(),
        if report.warnings.is_empty() { "none".to_string() } else { report.warnings.join("; ") }
    )
}

fn human_scene_report(report: &SceneInspection) -> String {
    format!(
        "scene: {}\nvalid: {}\ndocument: {}\nartboards: {}\nnodes: {}\ntimelines: {}\nstates: {}\nevents: {}\nwarnings: {}",
        report.scene_file,
        report.validation.ok,
        report.summary.name,
        report.summary.artboards,
        report.summary.nodes,
        report.summary.timelines,
        report.states.iter().map(|state| state.states.len()).sum::<usize>(),
        report.events.len(),
        if report.warnings.is_empty() { "none".to_string() } else { report.warnings.join("; ") }
    )
}

fn human_plan_report(plan: &CliPlanFile) -> String {
    let explanation = plan
        .explanation
        .as_ref()
        .map(|value| format!("\nexplain: {value}"))
        .unwrap_or_default();
    format!(
        "plan: {}\nbackend: {}\nsubject: {} ({})\nparts: {}\ntimelines: {}\nvalidated batch: {}{}",
        plan.instruction,
        plan.backend,
        plan.plan_summary.subject_label,
        plan.plan_summary.subject_classification,
        plan.plan_summary.part_names.len(),
        plan.plan_summary.timeline_names.join(", "),
        plan.batch.id,
        explanation
    )
}

fn instruction_kind(instruction: &str) -> String {
    let lower = instruction.to_lowercase();
    if lower.contains("logo") {
        "logo"
    } else if lower.contains("loader") || lower.contains("progress") || lower.contains("loading") {
        "loader"
    } else if lower.contains("mascot") || lower.contains("character") {
        "mascot"
    } else if lower.contains("icon") || lower.contains("badge") {
        "icon"
    } else if lower.contains("button") || lower.contains("microinteraction") || lower.contains("ui")
    {
        "ui"
    } else {
        "dice"
    }
    .to_string()
}

fn fixture_envelope(kind: &str) -> CliResult<Value> {
    let raw = match kind {
        "logo" => include_str!("../../../packages/strut-python/fixtures/logo.plan.json"),
        "loader" => include_str!("../../../packages/strut-python/fixtures/loader.plan.json"),
        "mascot" => include_str!("../../../packages/strut-python/fixtures/mascot.plan.json"),
        "ui" => include_str!("../../../packages/strut-python/fixtures/ui.plan.json"),
        "icon" | "badge" => include_str!("../../../packages/strut-python/fixtures/icon.plan.json"),
        _ => include_str!("../../../packages/strut-python/fixtures/dice.plan.json"),
    };
    Ok(serde_json::from_str(raw)?)
}

fn sprite_python_envelope(instruction: &str) -> CliResult<(Value, String, Vec<String>)> {
    let kind = instruction_kind(instruction);
    let package_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/strut-python");
    let output = Command::new("python")
        .arg("-m")
        .arg("strut_python.cli")
        .arg(&kind)
        .arg("--json")
        .current_dir(&package_dir)
        .env("PYTHONPATH", package_dir.join("src"))
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let envelope = serde_json::from_slice::<Value>(&output.stdout)?;
            Ok((envelope, "sprite-python".to_string(), Vec::new()))
        }
        Ok(output) => {
            let warning = format!(
                "sprite-python backend exited with {}; fell back to checked fixture: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            );
            Ok((
                fixture_envelope(&kind)?,
                "sprite-python-fixture".to_string(),
                vec![warning],
            ))
        }
        Err(error) => Ok((
            fixture_envelope(&kind)?,
            "sprite-python-fixture".to_string(),
            vec![format!(
                "sprite-python backend was unavailable ({error}); fell back to checked fixture"
            )],
        )),
    }
}

fn document_from_generation_plan_value(value: &Value) -> CliResult<PlannedDocument> {
    let envelope_value = if value.get("plan").is_some() {
        value.clone()
    } else {
        return Err(CliError::Message(
            "generation response must include a plan object".to_string(),
        ));
    };
    let envelope: GenerationPlanEnvelope = serde_json::from_value(envelope_value)
        .map_err(|error| CliError::Message(format!("generation plan schema mismatch: {error}")))?;
    validate_generation_plan(&envelope.plan)?;
    let operations = envelope.operations;
    validate_scene_operations(&envelope.plan, &operations)?;
    let document = document_from_scene_operations(&envelope.plan, &operations)?;
    strut_format::validate_document(&document)?;
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
        operations,
    })
}

fn validate_generation_plan(plan: &GenerationPlan) -> CliResult<()> {
    if plan.name.trim().is_empty() {
        return Err(CliError::Message(
            "generation plan name is required".to_string(),
        ));
    }
    if plan.parts.len() < 5 {
        return Err(CliError::Message(
            "generation plan must include at least five semantic parts".to_string(),
        ));
    }
    let mut part_ids = HashSet::new();
    let mut role_ids = HashSet::new();
    for role in &plan.motion_roles {
        if role.id.trim().is_empty() || role.purpose.trim().is_empty() {
            return Err(CliError::Message(
                "motion roles need non-empty id and purpose".to_string(),
            ));
        }
        if !role_ids.insert(role.id.as_str()) {
            return Err(CliError::Message(format!(
                "duplicate motion role id '{}'",
                role.id
            )));
        }
    }
    for part in &plan.parts {
        if part.id.trim().is_empty() || part.name.trim().is_empty() {
            return Err(CliError::Message(
                "semantic parts need non-empty id and name".to_string(),
            ));
        }
        if part.role.trim().is_empty() {
            return Err(CliError::Message(format!(
                "part '{}' must include a semantic role",
                part.id
            )));
        }
        if !part_ids.insert(part.id.as_str()) {
            return Err(CliError::Message(format!(
                "duplicate part id '{}'",
                part.id
            )));
        }
        if part
            .style
            .opacity
            .is_some_and(|opacity| !opacity.is_finite() || !(0.0..=1.0).contains(&opacity))
        {
            return Err(CliError::Message(format!(
                "part '{}' has invalid opacity",
                part.id
            )));
        }
        validate_part_geometry(part)?;
        if !part.constraints.editable && plan.editability.locked_parts.is_empty() {
            return Err(CliError::Message(format!(
                "part '{}' is non-editable but the plan did not list locked parts",
                part.id
            )));
        }
        for property in &part.constraints.allowed_properties {
            if !allowed_edit_property(property) {
                return Err(CliError::Message(format!(
                    "part '{}' allows unsupported editable property '{}'",
                    part.id, property
                )));
            }
        }
        for role in &part.motion_roles {
            if !role_ids.is_empty() && !role_ids.contains(role.as_str()) {
                return Err(CliError::Message(format!(
                    "part '{}' references missing motion role '{}'",
                    part.id, role
                )));
            }
        }
    }
    for role in &plan.motion_roles {
        for part_ref in &role.part_refs {
            if !part_ids.contains(part_ref.as_str()) {
                return Err(CliError::Message(format!(
                    "motion role '{}' references missing part '{}'",
                    role.id, part_ref
                )));
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
            return Err(CliError::Message(format!(
                "non-mascot subject '{}' cannot use mascot-only anatomy: {}",
                plan.subject.classification,
                mascot_parts.join(", ")
            )));
        }
    }
    let states = normalized_state_set(&plan.states);
    if !states.contains("idle") {
        return Err(CliError::Message(
            "generation plan must include an idle state".to_string(),
        ));
    }
    let mut timeline_ids = HashSet::new();
    for timeline in &plan.timelines {
        if !timeline_ids.insert(timeline.id.as_str()) {
            return Err(CliError::Message(format!(
                "duplicate timeline id '{}'",
                timeline.id
            )));
        }
        if timeline.duration_ms == 0 || timeline.tracks.is_empty() {
            return Err(CliError::Message(format!(
                "timeline '{}' needs duration and tracks",
                timeline.name
            )));
        }
        if let Some(state) = &timeline.state {
            if !states.contains(normalized_state_name(state).as_str()) {
                return Err(CliError::Message(format!(
                    "timeline '{}' references unknown state '{}'",
                    timeline.name, state
                )));
            }
        }
        for track in &timeline.tracks {
            if !part_ids.contains(track.target.as_str()) {
                return Err(CliError::Message(format!(
                    "timeline '{}' track targets missing part '{}'",
                    timeline.name, track.target
                )));
            }
            if !allowed_timeline_property(&track.property) || track.keyframes.len() < 2 {
                return Err(CliError::Message(format!(
                    "timeline '{}' has invalid track '{}'",
                    timeline.name, track.target
                )));
            }
            for keyframe in &track.keyframes {
                if keyframe.time_ms > timeline.duration_ms || !keyframe.value.is_finite() {
                    return Err(CliError::Message(format!(
                        "timeline '{}' has an invalid keyframe",
                        timeline.name
                    )));
                }
                if let Some(easing) = &keyframe.easing {
                    let _ = normalized_easing_name(easing);
                }
            }
        }
    }
    for editable_part in &plan.editability.editable_parts {
        if !part_ids.contains(editable_part.as_str()) {
            return Err(CliError::Message(format!(
                "editability references missing editable part '{}'",
                editable_part
            )));
        }
    }
    for locked_part in &plan.editability.locked_parts {
        if !part_ids.contains(locked_part.as_str()) {
            return Err(CliError::Message(format!(
                "editability references missing locked part '{}'",
                locked_part
            )));
        }
    }
    Ok(())
}

fn validate_part_geometry(part: &SemanticPartPlan) -> CliResult<()> {
    match part.geometry.kind.to_lowercase().as_str() {
        "rect" | "rectangle" => {
            let width = part.geometry.width.unwrap_or_default();
            let height = part.geometry.height.unwrap_or_default();
            if width <= 0.0 || height <= 0.0 || !width.is_finite() || !height.is_finite() {
                return Err(CliError::Message(format!(
                    "part '{}' has invalid rect geometry",
                    part.id
                )));
            }
        }
        "ellipse" => {
            let rx = part
                .geometry
                .rx
                .or_else(|| part.geometry.width.map(|width| width / 2.0))
                .unwrap_or_default();
            let ry = part
                .geometry
                .ry
                .or_else(|| part.geometry.height.map(|height| height / 2.0))
                .unwrap_or_default();
            if rx <= 0.0 || ry <= 0.0 || !rx.is_finite() || !ry.is_finite() {
                return Err(CliError::Message(format!(
                    "part '{}' has invalid ellipse geometry",
                    part.id
                )));
            }
        }
        "path" => {
            if part
                .geometry
                .d
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                return Err(CliError::Message(format!(
                    "part '{}' path geometry must include d",
                    part.id
                )));
            }
        }
        "text" => {
            let size = part.geometry.size.unwrap_or(24.0);
            if size <= 0.0 || !size.is_finite() {
                return Err(CliError::Message(format!(
                    "part '{}' text geometry has invalid size",
                    part.id
                )));
            }
        }
        other => {
            return Err(CliError::Message(format!(
                "part '{}' uses unsupported geometry kind '{}'",
                part.id, other
            )))
        }
    }
    Ok(())
}

fn validate_scene_operations(
    plan: &GenerationPlan,
    operations: &[SceneOperation],
) -> CliResult<()> {
    if operations.is_empty() {
        return Err(CliError::Message(
            "generation plan did not produce operations".to_string(),
        ));
    }
    let plan_parts = plan
        .parts
        .iter()
        .map(|part| part.id.as_str())
        .collect::<HashSet<_>>();
    let mut created_nodes = HashSet::new();
    let mut grouped_children = HashSet::new();
    let mut timelines = HashMap::<&str, u32>::new();
    for operation in operations {
        match operation {
            SceneOperation::CreateNode { id, geometry, .. } => {
                if !plan_parts.contains(id.as_str()) {
                    return Err(CliError::Message(format!(
                        "create_node references part outside plan: '{id}'"
                    )));
                }
                if !created_nodes.insert(id.as_str()) {
                    return Err(CliError::Message(format!(
                        "duplicate create_node id '{id}'"
                    )));
                }
                let part = SemanticPartPlan {
                    id: id.clone(),
                    name: id.clone(),
                    role: "operation".to_string(),
                    geometry: geometry.clone(),
                    style: PlanStyle::default(),
                    motion_roles: Vec::new(),
                    constraints: EditabilityConstraint::default(),
                };
                validate_part_geometry(&part)?;
            }
            SceneOperation::GroupNodes { children, .. } => {
                for child in children {
                    if !plan_parts.contains(child.as_str()) {
                        return Err(CliError::Message(format!(
                            "group_nodes references missing child '{child}'"
                        )));
                    }
                    grouped_children.insert(child.as_str());
                }
            }
            SceneOperation::AddTimeline {
                id, duration_ms, ..
            } => {
                timelines.insert(id.as_str(), *duration_ms);
            }
            SceneOperation::AddKeyframe {
                timeline,
                target,
                property,
                time_ms,
                value,
                ..
            } => {
                let duration = timelines.get(timeline.as_str()).ok_or_else(|| {
                    CliError::Message(format!(
                        "add_keyframe references missing timeline '{timeline}'"
                    ))
                })?;
                if !plan_parts.contains(target.as_str())
                    || *time_ms > *duration
                    || !value.is_finite()
                {
                    return Err(CliError::Message(
                        "add_keyframe references invalid target, time, or value".to_string(),
                    ));
                }
                if !allowed_timeline_property(property) {
                    return Err(CliError::Message(format!(
                        "add_keyframe uses unsupported property '{property}'"
                    )));
                }
            }
            SceneOperation::BindProperty {
                target, property, ..
            } => {
                if !plan_parts.contains(target.as_str()) || !allowed_edit_property(property) {
                    return Err(CliError::Message(
                        "bind_property references invalid target or property".to_string(),
                    ));
                }
            }
            SceneOperation::SetProperty {
                target, property, ..
            } => {
                if !plan_parts.contains(target.as_str()) || property.trim().is_empty() {
                    return Err(CliError::Message(
                        "set_property references invalid target or property".to_string(),
                    ));
                }
            }
            SceneOperation::AddState { state } => {
                if state.trim().is_empty() {
                    return Err(CliError::Message("add_state must not be empty".to_string()));
                }
            }
            SceneOperation::EmitEvent { name, .. } => {
                if name.trim().is_empty() {
                    return Err(CliError::Message(
                        "emit_event must not be empty".to_string(),
                    ));
                }
            }
        }
    }
    for part in &plan.parts {
        if !created_nodes.contains(part.id.as_str()) {
            return Err(CliError::Message(format!(
                "operations did not create planned part '{}'",
                part.id
            )));
        }
        if !grouped_children.contains(part.id.as_str()) {
            return Err(CliError::Message(format!(
                "operations did not group planned part '{}'",
                part.id
            )));
        }
    }
    Ok(())
}

fn document_from_scene_operations(
    plan: &GenerationPlan,
    operations: &[SceneOperation],
) -> CliResult<Document> {
    let mut semantic_ids = SemanticIdMap::default();
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
                        "id": semantic_ids.uuid_for(id),
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
                push_unique(&mut states, normalized_state_name(state))
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
                        "id": semantic_ids.uuid_for(id),
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
                        &semantic_ids.uuid_for(target),
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
            } => bindings.push(json!({
                "name": name,
                "target": semantic_ids.uuid_for(target),
                "property": normalize_bind_property(property)
            })),
            SceneOperation::EmitEvent { name, description } => {
                events.push(json!({ "name": name, "description": description }));
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
            "id": semantic_ids.uuid_for(&id),
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
            "id": semantic_ids.uuid_for("SceneRig"),
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
            let id_uuid = timeline.get("id").and_then(Value::as_str)?;
            let timeline_id = semantic_ids.semantic_for_uuid(id_uuid)?;
            let name = timeline.get("name").and_then(Value::as_str)?;
            let state = timeline_states
                .get(timeline_id)
                .and_then(Clone::clone)
                .unwrap_or_else(|| normalized_state_name(name));
            Some(json!({
                "from": "idle",
                "to": state,
                "on": state,
                "timeline": name
            }))
        })
        .collect::<Vec<_>>();
    let document_value = json!({
        "id": semantic_ids.uuid_for(plan.id.as_deref().unwrap_or("generation-plan-document")),
        "name": plan.name,
        "artboards": [{
            "id": semantic_ids.uuid_for("main-artboard"),
            "name": format!("{} Artboard", semantic_label(&plan.name)),
            "width": 960.0,
            "height": 540.0,
            "nodes": [root]
        }],
        "timelines": timeline_values,
        "state_machines": [{
            "id": semantic_ids.uuid_for("motion-machine"),
            "name": format!("{} Motion", semantic_label(&plan.subject.label)),
            "inputs": [{"name": "state", "kind": "enum"}],
            "states": states,
            "transitions": transitions
        }],
        "bindings": bindings,
        "events": events
    });
    serde_json::from_value(document_value)
        .map_err(|error| CliError::Message(format!("generated document schema mismatch: {error}")))
}

#[derive(Default)]
struct SemanticIdMap {
    by_semantic: HashMap<String, String>,
    by_uuid: HashMap<String, String>,
    next_id: u128,
}

impl SemanticIdMap {
    fn uuid_for(&mut self, semantic: &str) -> String {
        if let Some(id) = self.by_semantic.get(semantic) {
            return id.clone();
        }
        self.next_id += 1;
        let id = format!("00000000-0000-0000-0000-{:012x}", self.next_id);
        self.by_semantic.insert(semantic.to_string(), id.clone());
        self.by_uuid.insert(id.clone(), semantic.to_string());
        id
    }

    fn semantic_for_uuid(&self, id: &str) -> Option<&str> {
        self.by_uuid.get(id).map(String::as_str)
    }
}

fn plan_style_value(style: &PlanStyle) -> Value {
    let fill = style.fill.as_deref().unwrap_or("#f6f0df");
    json!({
        "fill": if fill.eq_ignore_ascii_case("none") || fill.eq_ignore_ascii_case("transparent") { Value::Null } else { json!(fill) },
        "stroke": style.stroke.as_deref().map_or(Value::Null, |stroke| json!(stroke)),
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
    if let Some(track) = tracks.iter_mut().find(|track| {
        track.get("target").and_then(Value::as_str) == Some(target)
            && track.get("property").and_then(Value::as_str) == Some(property)
    }) {
        if let Some(keyframes) = track.get_mut("keyframes").and_then(Value::as_array_mut) {
            keyframes.push(keyframe_value(time_ms, value, easing));
        }
        return;
    }
    tracks.push(json!({
        "target": target,
        "property": property,
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
        .collect()
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
        .collect()
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn document_revision_id(document: &Document) -> String {
    format!(
        "rev-{}-{}-{}-{}",
        document.name.to_lowercase().replace(' ', "-"),
        document.artboards.len(),
        count_document_nodes(document),
        document.timelines.len()
    )
}

fn validate_operation_batch(batch: &OperationBatchRecord) -> CliResult<()> {
    if batch.id.trim().is_empty() {
        return Err(CliError::Message(
            "operation batch id is required".to_string(),
        ));
    }
    if !matches!(
        batch.source_type.as_str(),
        "ai" | "sprite-python" | "manual" | "cli"
    ) {
        return Err(CliError::Message(format!(
            "operation batch '{}' has unsupported source type '{}'",
            batch.id, batch.source_type
        )));
    }
    if !matches!(
        batch.status.as_str(),
        "pending" | "applied" | "rejected" | "undone"
    ) {
        return Err(CliError::Message(format!(
            "operation batch '{}' has unsupported status '{}'",
            batch.id, batch.status
        )));
    }
    if !batch.document_revision_id.starts_with("rev-") {
        return Err(CliError::Message(format!(
            "operation batch '{}' has unsupported document revision id '{}'",
            batch.id, batch.document_revision_id
        )));
    }
    if matches!(batch.status.as_str(), "pending" | "applied" | "undone")
        && batch.operations.is_empty()
    {
        return Err(CliError::Message(format!(
            "operation batch '{}' has no meaningful operations",
            batch.id
        )));
    }
    if batch.status == "applied" && !batch.validation_result.ok {
        return Err(CliError::Message(format!(
            "operation batch '{}' cannot be applied with failed validation",
            batch.id
        )));
    }
    for operation in &batch.operations {
        match operation.get("type").and_then(Value::as_str) {
            Some("replace_document") => {
                let next_document = operation.get("nextDocument").ok_or_else(|| {
                    CliError::Message(format!(
                        "operation batch '{}' replace_document operation needs nextDocument",
                        batch.id
                    ))
                })?;
                let document: Document = serde_json::from_value(next_document.clone())?;
                strut_format::validate_document(&document)?;
                if let Some(previous_document) = operation.get("previousDocument") {
                    if !previous_document.is_null() {
                        let previous: Document = serde_json::from_value(previous_document.clone())?;
                        strut_format::validate_document(&previous)?;
                    }
                }
            }
            Some(other) => {
                return Err(CliError::Message(format!(
                    "operation batch '{}' contains unsupported CLI patch operation '{}'",
                    batch.id, other
                )));
            }
            None => {
                return Err(CliError::Message(format!(
                    "operation batch '{}' contains a malformed operation",
                    batch.id
                )));
            }
        }
    }
    Ok(())
}

fn render_svg_proof(document: &Document, state: &str) -> String {
    let artboard = &document.artboards[0];
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" role="img" aria-label="Strut render proof for {}">"#,
        artboard.width, artboard.height, artboard.width, artboard.height, document.name
    );
    svg.push_str(r##"<rect width="100%" height="100%" fill="#f8fafc"/>"##);
    svg.push_str(&format!(
        r##"<text x="24" y="36" font-family="Arial, sans-serif" font-size="18" fill="#0f172a">{} / state: {}</text>"##,
        escape_xml(&document.name),
        escape_xml(state)
    ));
    for node in &artboard.nodes {
        append_node_svg(&mut svg, node);
    }
    svg.push_str("</svg>\n");
    svg
}

fn append_node_svg(svg: &mut String, node: &strut_core::Node) {
    match &node.shape {
        strut_core::Shape::Rect {
            x,
            y,
            width,
            height,
            rx,
        } => svg.push_str(&format!(
            r#"<rect x="{x}" y="{y}" width="{width}" height="{height}" rx="{rx}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"/>"#,
            paint(node.style.fill.as_deref()),
            paint(node.style.stroke.as_deref()),
            node.style.stroke_width,
            node.style.opacity
        )),
        strut_core::Shape::Ellipse { cx, cy, rx, ry } => svg.push_str(&format!(
            r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"/>"#,
            paint(node.style.fill.as_deref()),
            paint(node.style.stroke.as_deref()),
            node.style.stroke_width,
            node.style.opacity
        )),
        strut_core::Shape::Path { d } => svg.push_str(&format!(
            r#"<path d="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
            escape_xml(d),
            paint(node.style.fill.as_deref()),
            paint(node.style.stroke.as_deref()),
            node.style.stroke_width,
            node.style.opacity
        )),
        strut_core::Shape::Text { x, y, value, size } => svg.push_str(&format!(
            r#"<text x="{x}" y="{y}" font-family="Arial, sans-serif" font-size="{size}" fill="{}" opacity="{}">{}</text>"#,
            paint(node.style.fill.as_deref()),
            node.style.opacity,
            escape_xml(value)
        )),
        strut_core::Shape::None => {}
    }
    for child in &node.children {
        append_node_svg(svg, child);
    }
}

fn paint(value: Option<&str>) -> &str {
    value.unwrap_or("none")
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn react_export_files(document: &Document) -> Vec<(PathBuf, String)> {
    let scene_json = serde_json::to_string_pretty(document).expect("document serializes");
    let component = format!(
        r#"import scene from "./scene.json";

type StrutNode = {{
  id: string;
  name: string;
  kind: string;
  role?: string;
  shape: {{ type: string; [key: string]: unknown }};
  style: {{ fill?: string | null; stroke?: string | null; stroke_width?: number; opacity?: number }};
  children?: StrutNode[];
}};

function paint(value: string | null | undefined) {{
  return value ?? "none";
}}

function renderNode(node: StrutNode): React.ReactNode {{
  const style = node.style ?? {{}};
  const common = {{
    key: node.id,
    fill: paint(style.fill),
    stroke: paint(style.stroke),
    strokeWidth: style.stroke_width ?? 0,
    opacity: style.opacity ?? 1,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    "data-strut-node": node.name,
    "data-strut-role": node.role ?? "",
  }};
  const shape = node.shape ?? {{ type: "none" }};
  if (shape.type === "rect") {{
    return <rect {{...common}} x={{shape.x as number}} y={{shape.y as number}} width={{shape.width as number}} height={{shape.height as number}} rx={{shape.rx as number}} />;
  }}
  if (shape.type === "ellipse") {{
    return <ellipse {{...common}} cx={{shape.cx as number}} cy={{shape.cy as number}} rx={{shape.rx as number}} ry={{shape.ry as number}} />;
  }}
  if (shape.type === "path") {{
    return <path {{...common}} d={{shape.d as string}} />;
  }}
  if (shape.type === "text") {{
    return <text {{...common}} x={{shape.x as number}} y={{shape.y as number}} fontSize={{shape.size as number}}>{{shape.value as string}}</text>;
  }}
  return <g key={{node.id}}>{{node.children?.map(renderNode)}}</g>;
}}

export function StrutAnimation({{ state = "idle", title = "{}" }}: {{ state?: string; title?: string }}) {{
  const artboard = scene.artboards[0];
  return (
    <svg viewBox={{`0 0 ${{artboard.width}} ${{artboard.height}}`}} role="img" aria-label={{title}} data-strut-state={{state}}>
      {{artboard.nodes.map(renderNode)}}
    </svg>
  );
}}

export default StrutAnimation;
"#,
        document.name.replace('"', "\\\"")
    );
    let readme = format!(
        "# Strut React Export\n\nGenerated from `{}`.\n\n```tsx\nimport {{ StrutAnimation }} from \"./StrutAnimation\";\n\nexport function Example() {{\n  return <StrutAnimation state=\"idle\" />;\n}}\n```\n\nThe component renders the validated `.strut` document as static SVG markup. Runtime timeline playback is a future runtime integration layer.\n",
        document.name
    );
    vec![
        (PathBuf::from("scene.json"), scene_json),
        (PathBuf::from("StrutAnimation.tsx"), component),
        (PathBuf::from("README.md"), readme),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_plan_compiles_to_valid_document() {
        let envelope = fixture_envelope("dice").expect("fixture");
        let planned = document_from_generation_plan_value(&envelope).expect("planned");
        assert_eq!(planned.summary.subject_classification, "dice");
        assert!(count_document_nodes(&planned.document) >= 6);
        strut_format::validate_document(&planned.document).expect("valid document");
    }

    #[test]
    fn non_mascot_fixture_rejects_mascot_parts_if_tampered() {
        let mut envelope = fixture_envelope("logo").expect("fixture");
        envelope["plan"]["parts"][0]["name"] = json!("Head");
        let error = document_from_generation_plan_value(&envelope).expect_err("rejected");
        assert!(error.to_string().contains("mascot-only anatomy"));
    }

    #[test]
    fn render_proof_contains_scene_name() {
        let document = Document::sample_login_button();
        let svg = render_svg_proof(&document, "idle");
        assert!(svg.contains("Login Button"));
        assert!(svg.contains("<svg"));
    }
}
