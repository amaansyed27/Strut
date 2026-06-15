use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalGenerationKind {
    OllamaHttp,
    SpritePython,
    StdinPrompt,
    AcpOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestIntent {
    Conversation,
    Generate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationStrategy {
    SimpleSvg,
    SpritePython,
    ProviderPlan,
}

#[derive(Debug, Clone)]
pub struct LocalAdapterDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    pub commands: &'static [&'static str],
    pub version_args: &'static [&'static str],
    pub generation: LocalGenerationKind,
}

#[derive(Debug)]
pub struct CommandRun {
    pub ok: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentAdapterStatus {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub command: Option<String>,
    pub installed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ByokProviderConfig {
    pub provider_id: String,
    pub api_key: Option<String>,
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationProvider {
    pub mode: String,
    pub local_adapter_id: Option<String>,
    pub byok: Option<ByokProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceImageInput {
    pub name: String,
    pub mime_type: String,
    pub data_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationContext {
    pub project_name: Option<String>,
    pub project_path: Option<String>,
    pub active_chat_title: Option<String>,
    pub current_document_summary: Option<String>,
    pub chat_history: Vec<GenerationContextMessage>,
    pub current_document: Option<strut_core::Document>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationContextMessage {
    pub role: String,
    pub text: String,
    pub attachments: Option<Vec<String>>,
}

pub const ASSISTANT_ROUTER_SYSTEM_PROMPT: &str = r#"You are the Strut generation router. The user will provide a prompt. You must output exactly ONE valid JSON object and nothing else. The JSON object must match this schema:
{
    "kind": "chat",
    "message": "Your response message"
}
OR
{
    "kind": "document_created",
    "message": "A summary of what you created",
    "document": {
        "plan": <GenerationPlan>,
        "operations": <SceneOperation[]>
    }
}
OR
{
    "kind": "document_updated",
    "message": "A summary of what you updated",
    "document": {
        "plan": <GenerationPlan>,
        "operations": <SceneOperation[]>
    }
}
Do not use markdown blocks around the JSON.

"#;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssistantResult {
    Chat {
        message: String,
        #[serde(default)]
        source: String,
    },
    DocumentCreated {
        message: String,
        #[serde(default)]
        source: String,
        document: strut_core::Document,
    },
    DocumentUpdated {
        message: String,
        #[serde(default)]
        source: String,
        document: strut_core::Document,
    },
}

#[derive(Debug)]
pub struct WrittenReferenceFiles {
    pub directory: PathBuf,
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderOperationResult {
    pub ok: bool,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedCharacter {
    pub document: strut_core::Document,
    pub source: String,
    pub message: String,
    pub plan_summary: Option<GenerationPlanSummary>,
    pub operation_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAnswer {
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationPlanSummary {
    pub subject_classification: String,
    pub subject_label: String,
    pub part_names: Vec<String>,
    pub timeline_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PlannedDocument {
    pub document: strut_core::Document,
    pub summary: GenerationPlanSummary,
    }

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationPlanEnvelope {
    pub plan: GenerationPlan,
    #[serde(default)]
    pub operations: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationPlan {
    pub id: Option<String>,
    pub name: String,
    pub subject: SubjectPlan,
    #[serde(default)]
    pub parts: Vec<SemanticPartPlan>,
    #[serde(default, alias = "motion_roles")]
    pub motion_roles: Vec<MotionRolePlan>,
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub timelines: Vec<TimelinePlan>,
    #[serde(default)]
    pub editability: EditabilityPlan,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectPlan {
    pub classification: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticPartPlan {
    pub id: String,
    pub name: String,
    pub role: String,
    pub geometry: PlanGeometry,
    #[serde(default)]
    pub style: PlanStyle,
    #[serde(default, alias = "motion_roles")]
    pub motion_roles: Vec<String>,
    #[serde(default)]
    pub constraints: EditabilityConstraint,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanGeometry {
    pub kind: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub rx: Option<f64>,
    pub ry: Option<f64>,
    pub cx: Option<f64>,
    pub cy: Option<f64>,
    pub d: Option<String>,
    pub value: Option<String>,
    pub size: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStyle {
    pub fill: Option<String>,
    pub stroke: Option<String>,
    #[serde(alias = "stroke_width")]
    pub stroke_width: Option<f64>,
    pub opacity: Option<f64>,
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
pub struct EditabilityConstraint {
    #[serde(default = "default_editable")]
    pub editable: bool,
    #[serde(default, alias = "allowed_properties")]
    pub allowed_properties: Vec<String>,
}

impl Default for EditabilityConstraint {
    fn default() -> Self {
        Self {
            editable: true,
            allowed_properties: Vec::new(),
        }
    }
}

pub fn default_editable() -> bool {
    true
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditabilityPlan {
    #[serde(default, alias = "editable_parts")]
    pub editable_parts: Vec<String>,
    #[serde(default, alias = "locked_parts")]
    pub locked_parts: Vec<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotionRolePlan {
    pub id: String,
    pub purpose: String,
    #[serde(default, alias = "part_refs")]
    pub part_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePlan {
    pub id: String,
    pub name: String,
    pub state: Option<String>,
    #[serde(alias = "duration_ms")]
    pub duration_ms: u32,
    #[serde(default)]
    pub tracks: Vec<TimelineTrackPlan>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineTrackPlan {
    pub target: String,
    pub property: String,
    #[serde(default)]
    pub keyframes: Vec<KeyframePlan>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyframePlan {
    #[serde(alias = "t")]
    #[serde(alias = "time_ms")]
    #[serde(alias = "time")]
    pub time_ms: u32,
    #[serde(alias = "v")]
    #[serde(alias = "value")]
    pub value: f64,
    pub easing: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SceneOperation {
    CreateNode {
        id: String,
        name: String,
        kind: String,
        #[allow(dead_code)]
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
pub struct SavedByokProviderConfig {
    pub provider_id: String,
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub files: Vec<ProjectFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFile {
    pub name: String,
    pub path: String,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationValidationResult {
    pub ok: bool,
    pub message: String,
    pub validator: String,
    pub validated_at: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationBatchRecord {
    pub id: String,
    pub source_type: String,
    pub status: String,
    pub validation_result: OperationValidationResult,
    pub document_revision_id: String,
    pub previous_document_revision_id: Option<String>,
    pub prompt: Option<String>,
    pub source_metadata: Option<Value>,
    pub operations: Vec<Value>,
    pub created_at: String,
    pub updated_at: String,
    pub applied_at: Option<String>,
    pub rejected_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedSelectionState {
    pub active_state: String,
    pub selected_node_id: Option<String>,
    pub layer_ui: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub project: ProjectInfo,
    pub document: strut_core::Document,
    pub operation_batches: Vec<OperationBatchRecord>,
    pub selection: Option<PersistedSelectionState>,
    pub main_scene: String,
    pub animations: Vec<ProjectAnimationRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAnimationRecord {
    pub id: String,
    pub name: String,
    pub chat_id: Option<String>,
    pub scene: String,
    pub operation_batches: Vec<OperationBatchRecord>,
    pub selection: Option<PersistedSelectionState>,
    pub document: strut_core::Document,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatedGeneratedBatch {
    pub document: strut_core::Document,
    pub batch: OperationBatchRecord,
    }
