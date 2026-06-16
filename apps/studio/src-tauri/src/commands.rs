use serde_json::{json, Value};
use std::path::PathBuf;
use crate::*;

fn generation_debug_enabled() -> bool {
    std::env::var("STRUT_DEBUG_GENERATION")
        .ok()
        .is_some_and(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
}

fn debug_generation(label: &str, detail: impl AsRef<str>) {
    if generation_debug_enabled() {
        eprintln!("[strut:generation] {label}: {}", detail.as_ref());
    }
}

fn provider_debug_label(provider: &GenerationProvider) -> String {
    match provider.mode.as_str() {
        "local" => format!(
            "local adapter={}",
            provider.local_adapter_id.as_deref().unwrap_or("<missing>")
        ),
        "byok" => provider
            .byok
            .as_ref()
            .map(|config| format!("byok provider={} model={}", config.provider_id, config.model))
            .unwrap_or_else(|| "byok <missing config>".to_string()),
        other => other.to_string(),
    }
}
#[tauri::command]
pub fn studio_status() -> strut_format::StudioStatus {
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
pub fn default_project_location() -> String {
    default_projects_dir().display().to_string()
}

pub const PROJECT_MANIFEST_FILE: &str = "strut.project.json";
pub const MAIN_SCENE_FILE: &str = "scenes/main.strut";
pub const LEGACY_STARTER_SCENE_FILE: &str = "scenes/starter.strut.json";
pub const OPERATION_BATCHES_FILE: &str = "operations/operation-batches.json";
pub const STUDIO_STATE_FILE: &str = "ui/studio-state.json";
pub const ANIMATION_SCENE_DIR: &str = "scenes/animations";
pub const ANIMATION_OPERATION_DIR: &str = "operations/animations";

#[tauri::command]
pub fn create_project(name: String, location: String) -> Result<ProjectInfo, String> {
    let project_name = sanitize_project_name(&name)?;
    let root = if location.trim().is_empty() {
        default_projects_dir()
    } else {
        PathBuf::from(location.trim())
    };
    let project_path = root.join(&project_name);

    fs::create_dir_all(project_path.join("scenes")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join(ANIMATION_SCENE_DIR))
        .map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join("assets")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join("exports")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join("operations")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join(ANIMATION_OPERATION_DIR))
        .map_err(|error| error.to_string())?;
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
pub fn save_project_snapshot(
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
    let animations = read_project_animation_records(&root)?;
    let animation_entries = animations
        .iter()
        .map(project_animation_manifest_entry)
        .collect::<Vec<_>>();

    fs::create_dir_all(root.join("scenes")).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join(ANIMATION_SCENE_DIR)).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join("operations")).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join(ANIMATION_OPERATION_DIR)).map_err(|error| error.to_string())?;
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
        serde_json::to_string_pretty(&project_manifest_value_with_animations(
            &project_name,
            unix_timestamp(),
            animation_entries,
        ))
        .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(ProjectSnapshot {
        project: project_info(project_name, root),
        document,
        operation_batches,
        selection,
        main_scene: MAIN_SCENE_FILE.to_string(),
        animations,
    })
}

#[tauri::command]
pub fn save_project_animation(
    project_path: String,
    project_name: String,
    chat_id: String,
    animation_name: String,
    document: strut_core::Document,
    operation_batches: Vec<OperationBatchRecord>,
    selection: Option<PersistedSelectionState>,
) -> Result<ProjectAnimationRecord, String> {
    let root = ensure_project_root(&project_path)?;
    let project_name = sanitize_project_name(&project_name)?;
    let animation_name = sanitize_animation_name(&animation_name)?;
    strut_format::validate_document(&document).map_err(|error| error.to_string())?;
    validate_operation_batches(&operation_batches, &document)?;

    fs::create_dir_all(root.join(ANIMATION_SCENE_DIR)).map_err(|error| error.to_string())?;
    fs::create_dir_all(root.join(ANIMATION_OPERATION_DIR)).map_err(|error| error.to_string())?;

    let id = format!(
        "anim-{}-{}-{}",
        sanitize_token(&chat_id),
        sanitize_token(&animation_name),
        unix_timestamp()
    );
    let scene = format!("{ANIMATION_SCENE_DIR}/{id}.strut");
    let operation_path = format!("{ANIMATION_OPERATION_DIR}/{id}.json");
    let updated_at = unix_timestamp();

    strut_format::write_strut_file(
        root.join(&scene),
        &strut_format::StrutPackage::current(document.clone()),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        root.join(&operation_path),
        serde_json::to_string_pretty(&operation_batches).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let record = ProjectAnimationRecord {
        id,
        name: animation_name,
        chat_id: if chat_id.trim().is_empty() { None } else { Some(chat_id) },
        scene,
        operation_batches,
        selection,
        document,
        updated_at,
    };
    let mut animations = read_project_animation_records(&root)?;
    let replaced = animations
        .iter()
        .filter(|animation| {
            animation.id == record.id
                || (animation.chat_id == record.chat_id && animation.name == record.name)
        })
        .cloned()
        .collect::<Vec<_>>();
    animations.retain(|animation| {
        animation.id != record.id
            && !(animation.chat_id == record.chat_id && animation.name == record.name)
    });
    for old in replaced {
        let _ = fs::remove_file(root.join(&old.scene));
        if let Some(path) = project_animation_operation_path(&old) {
            let _ = fs::remove_file(root.join(path));
        }
    }
    animations.insert(0, record.clone());
    write_project_manifest_with_animation_records(&root, &project_name, &animations)?;
    Ok(record)
}

#[tauri::command]
pub fn delete_project_animation(project_path: String, animation_id: String) -> Result<(), String> {
    let root = ensure_project_root(&project_path)?;
    let manifest = read_project_manifest(&root)?;
    let project_name = project_name_from_manifest(&manifest, &root);
    let mut animations = read_project_animation_records(&root)?;
    let Some(removed) = animations.iter().find(|animation| animation.id == animation_id).cloned() else {
        return Err(format!("animation '{animation_id}' was not found in this project"));
    };

    animations.retain(|animation| animation.id != animation_id);
    let scene_path = safe_project_file_path(&root, &removed.scene, "animation scene")?;
    if scene_path.exists() {
        fs::remove_file(&scene_path).map_err(|error| error.to_string())?;
    }
    let operation_path = project_animation_operation_path(&removed);
    if let Some(operation_path) = operation_path {
        let path = safe_project_file_path(&root, &operation_path, "animation operation batches")?;
        if path.exists() {
            fs::remove_file(&path).map_err(|error| error.to_string())?;
        }
    }
    write_project_manifest_with_animation_records(&root, &project_name, &animations)?;
    Ok(())
}

#[tauri::command]
pub fn load_project_snapshot(project_path: String) -> Result<ProjectSnapshot, String> {
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
    let animations = read_project_animation_records(&root)?;

    Ok(ProjectSnapshot {
        project: project_info(project_name, root),
        document,
        operation_batches,
        selection,
        main_scene,
        animations,
    })
}

#[tauri::command]
pub fn validate_scene_document(document: strut_core::Document) -> OperationValidationResult {
    validation_result(strut_format::validate_document(&document).map_err(|error| error.to_string()))
}

#[tauri::command]
pub fn validate_generation_plan_batch(
    source_text: String,
    source_type: String,
    prompt: Option<String>,
) -> Result<ValidatedGeneratedBatch, String> {
    let document = document_from_generation_plan_text(&source_text)?;
    let operations = operation_values_from_generation_plan_text(&source_text);
    let timestamp = timestamp_label();
    let revision = document_revision_id(&document);
    let batch = OperationBatchRecord {
        id: format!(
            "batch-{}-{}-{}",
            sanitize_token(&source_type),
            revision,
            0
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
            "subjectClassification": "object".to_string(),
            "subjectLabel": "generated".to_string(),
            "operationCount": 0
        })),
        operations,
        created_at: timestamp.clone(),
        updated_at: timestamp.clone(),
        applied_at: Some(timestamp),
        rejected_at: None,
    };

    Ok(ValidatedGeneratedBatch {
        document: document.clone(),
        batch,
        
    })
}

#[tauri::command]
pub fn open_project_folder(path: String) -> Result<(), String> {
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
async fn call_provider_for_assistant(
    prompt: &str,
    provider: &GenerationProvider,
    references: &[ReferenceImageInput],
    system_prompt: &str,
) -> Result<String, String> {
    match provider.mode.as_str() {
        "byok" => {
            let config = provider
                .byok
                .as_ref()
                .ok_or_else(|| "BYOK provider config missing".to_string())?;
            byok_generate_text(prompt, config, references, Some(system_prompt)).await
        }
        "local" => {
            let adapter_id = provider
                .local_adapter_id
                .as_ref()
                .ok_or_else(|| "Select a local CLI or Ollama adapter".to_string())?;
            let raw_text = chat_with_local_adapter(adapter_id, prompt, references, system_prompt).await?;
            Ok(cli_assistant_text(&raw_text))
        }
        _ => Err("Unknown provider mode".to_string()),
    }
}

#[tauri::command]
pub async fn assistant_message(
    prompt: String,
    provider: Option<GenerationProvider>,
    references: Option<Vec<ReferenceImageInput>>,
    context: Option<GenerationContext>,
) -> Result<AssistantResult, String> {
    let references = references.unwrap_or_default();
    let provider = provider.ok_or_else(|| {
        "Select a real local CLI, Ollama, or BYOK provider before generating.".to_string()
    })?;

    let mut system_prompt = format!("{}\n\n{}", ASSISTANT_ROUTER_SYSTEM_PROMPT, GENERATION_PLAN_SYSTEM_PROMPT);
    if let Some(context) = context.as_ref() {
        if let Some(project_name) = &context.project_name {
            system_prompt.push_str(&format!("\nProject: {project_name}"));
        }
        if let Some(chat_title) = &context.active_chat_title {
            system_prompt.push_str(&format!("\nChat: {chat_title}"));
        }
        if let Some(summary) = &context.current_document_summary {
            system_prompt.push_str(&format!("\n\nThe scene currently contains this document:\n{summary}"));
        }
    }

    let user_prompt = prompt.clone();
    let request_intent = classify_request_intent(&user_prompt);
    let route_to_chat = should_route_to_chat_response(&user_prompt, context.as_ref());
    debug_generation("classification", format!("{request_intent:?}"));
    debug_generation("chat_early_exit", route_to_chat.to_string());
    debug_generation("system_prompt_preview", response_preview(&system_prompt));
    debug_generation("user_prompt", &user_prompt);
    debug_generation("provider", provider_debug_label(&provider));

    if route_to_chat {
        let chat_prompt = chat_system_prompt(&user_prompt, context.as_ref());
        debug_generation("chat_prompt_preview", response_preview(&chat_prompt));
        let text = call_provider_for_assistant(&user_prompt, &provider, &references, &chat_prompt).await?;
        debug_generation("llm_response_preview", response_preview(&text));
        debug_generation("final_result", "chat");
        return Ok(AssistantResult::Chat {
            message: text,
            source: "llm".to_string(),
        });
    }

    let text = call_provider_for_assistant(&user_prompt, &provider, &references, &system_prompt).await?;
    debug_generation("llm_response_preview", response_preview(&text));

    // Generation mode: attempt to parse as Strut document
    // Chat mode requests already returned early above, so we know this is generation
    let initial_result = match parse_assistant_result_from_text(&text) {
        Ok(result) => {
            debug_generation("parse_initial", "ok");
            result
        }
        Err(first_error) => {
            debug_generation("parse_initial", format!("error: {first_error}"));
            // Apply repair/compact plan fallback for generation mode only
            // No need to check intent again - chat mode already returned early
            let repair_prompt = generation_plan_repair_prompt(&user_prompt, &text, &first_error);
            let repair_text = match call_provider_for_assistant(&repair_prompt, &provider, &references, &system_prompt).await {
                Ok(t) => t,
                Err(e) => return Err(format!("Repair generation failed: {}", e)),
            };
            debug_generation("repair_response_preview", response_preview(&repair_text));

            match parse_assistant_result_from_text(&repair_text) {
                Ok(result) => {
                    debug_generation("parse_repair", "ok");
                    result
                }
                Err(repair_error) => {
                    debug_generation("parse_repair", format!("error: {repair_error}"));
                    let compact_prompt = compact_plan_prompt(&user_prompt, &repair_error);
                    let compact_text = match call_provider_for_assistant(&compact_prompt, &provider, &references, &system_prompt).await {
                        Ok(t) => t,
                        Err(e) => return Err(format!("Compact plan generation failed: {}", e)),
                    };
                    debug_generation("compact_response_preview", response_preview(&compact_text));

                    match parse_assistant_result_from_text(&compact_text) {
                        Ok(result) => {
                            debug_generation("parse_compact", "ok");
                            result
                        }
                        Err(plan_error) => {
                            debug_generation("parse_compact", format!("error: {plan_error}"));
                            return Err(format!(
                                "Provider did not return valid Strut animation JSON after 3 attempts. Try a different provider or check provider configuration.\nFirst error: {first_error}\nRepair error: {repair_error}\nPlan error: {plan_error}\nResponse preview: {}",
                                response_preview(&compact_text)
                            ));
                        }
                    }
                }
            }
        }
    };

    // Quality reflection pass - DISABLED due to self-correction blind spot
    // Research shows LLMs reviewing their own output share the same biases
    // and can over-correct to blank/broken results. The reflection either
    // misses the same errors (too lenient) or rejects valid output (too strict).
    // See: arxiv.org/abs/2507.02778 - Self-Correction Blind Spot
    // TODO: Implement external validator or different sampling strategy
    /*
    if classify_request_intent(&user_prompt) == RequestIntent::Generate {
        if let AssistantResult::DocumentCreated { document, .. } | AssistantResult::DocumentUpdated { document, .. } = &initial_result {
            if let Ok(document_json) = serde_json::to_string_pretty(document) {
                let reflection_prompt = visual_quality_reflection_prompt(&user_prompt, &document_json);
                match call_provider_for_assistant(&reflection_prompt, &provider, &references, &system_prompt).await {
                    Ok(feedback_text) => {
                        match parse_assistant_result_from_text(&feedback_text) {
                            Ok(feedback_result) => {
                                // Successfully improved the document through reflection
                                initial_result = feedback_result;
                            }
                            Err(parse_error) => {
                                // Log the reflection failure but don't block the initial result
                                eprintln!("Quality reflection produced unparseable output: {parse_error}\nResponse preview: {}", response_preview(&feedback_text));
                            }
                        }
                    }
                    Err(call_error) => {
                        eprintln!("Quality reflection call failed: {call_error}");
                    }
                }
            }
        }
    }
    */

    debug_generation(
        "final_result",
        match &initial_result {
            AssistantResult::Chat { .. } => "chat",
            AssistantResult::DocumentCreated { .. } => "document_created",
            AssistantResult::DocumentUpdated { .. } => "document_updated",
        },
    );

    Ok(initial_result)
}
pub fn parse_assistant_result(json_str: &str) -> Result<AssistantResult, String> {
    let value = serde_json::from_str::<Value>(json_str.trim()).map_err(|e| e.to_string())?;
    parse_assistant_result_value(value)
}



#[tauri::command]
pub async fn test_byok_provider(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
    ensure_byok_config(&config)?;
    let smoke_prompt =
        "Create a small floating helper animation named Smoke Bot with a cyan accent.";
    match byok_generate_text(smoke_prompt, &config, &[], None).await {
        Ok(_) => Ok(ProviderOperationResult {
            ok: true,
            status: format!("{} ready", provider_label(&config.provider_id)),
            detail: "provider completed a real Strut generation smoke test".to_string(),
        }),
        Err(error) => Ok(ProviderOperationResult {
            ok: false,
            status: format!("{} failed smoke test", provider_label(&config.provider_id)),
            detail: error,
        }),
    }
}

#[tauri::command]
pub fn save_byok_provider(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
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

pub(crate) fn parse_assistant_result_from_text(text: &str) -> Result<AssistantResult, String> {
    let mut last_err = "Model response did not contain a recognizable chat or document payload.".to_string();

    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        match parse_assistant_result_value(value) {
            Ok(result) => return Ok(result),
            Err(e) => last_err = e,
        }
    }

    for json_text in extract_json_objects(text).into_iter().rev() {
        if let Ok(value) = serde_json::from_str::<Value>(&json_text) {
            match parse_assistant_result_value(value) {
                Ok(result) => return Ok(result),
                Err(e) => last_err = e,
            }
        }
    }
    
    match parsing::parse_generated_document(text) {
        Ok(doc) => {
            return Ok(AssistantResult::DocumentCreated {
                message: "Generated document from implicit plan.".to_string(),
                source: "llm".to_string(),
                document: doc,
                plan_summary: None,
                operation_count: None,
            });
        }
        Err(e) => {
            last_err = e;
        }
    }

    Err(last_err)
}

pub fn parse_assistant_result_value(value: Value) -> Result<AssistantResult, String> {
    if let Some(kind) = value.get("kind").and_then(|v| v.as_str()) {
        let message = value.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
        match kind {
            "chat" => {
                return Ok(AssistantResult::Chat {
                    message,
                    source: "llm".to_string(),
                });
            }
            "document_created" | "document_updated" => {
                let document_value = value.get("document").ok_or_else(|| "Missing document field".to_string())?;
                let plan_summary = generation_plan_summary_from_value(document_value);
                let operation_count = document_value
                    .get("operations")
                    .and_then(Value::as_array)
                    .map(Vec::len);
                let planned_doc = document_from_generation_plan_value(document_value)?;
                if kind == "document_created" {
                    return Ok(AssistantResult::DocumentCreated {
                        message,
                        source: "llm".to_string(),
                        document: planned_doc,
                        plan_summary,
                        operation_count,
                    });
                } else {
                    return Ok(AssistantResult::DocumentUpdated {
                        message,
                        source: "llm".to_string(),
                        document: planned_doc,
                        plan_summary,
                        operation_count,
                    });
                }
            }
            _ => return Err(format!("Unknown AssistantResult kind: {}", kind)),
        }
    }
    Err("Missing AssistantResult kind".to_string())
}

fn generation_plan_summary_from_value(value: &Value) -> Option<GenerationPlanSummary> {
    let plan = if let Some(plan) = value.get("plan") {
        plan
    } else if let Some(document_plan) = value.get("document").and_then(|document| document.get("plan")) {
        document_plan
    } else {
        return None;
    };

    let subject = plan.get("subject")?;
    let subject_classification = subject
        .get("classification")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let subject_label = subject
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let part_names = plan
        .get("parts")
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part.get("name").and_then(Value::as_str).or_else(|| part.get("id").and_then(Value::as_str)))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let timeline_names = plan
        .get("timelines")
        .and_then(Value::as_array)
        .map(|timelines| {
            timelines
                .iter()
                .filter_map(|timeline| timeline.get("name").and_then(Value::as_str).or_else(|| timeline.get("id").and_then(Value::as_str)))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(GenerationPlanSummary {
        subject_classification,
        subject_label,
        part_names,
        timeline_names,
    })
}


#[tauri::command]
pub fn local_agent_adapters() -> Vec<AgentAdapterStatus> {
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
                (Some(path), LocalGenerationKind::SpritePython) => format!(
                    "built-in sprite-python engine available through {} at {}",
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
pub fn test_local_adapter(adapter_id: String) -> ProviderOperationResult {
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
            &definition,
            GenerationStrategy::ProviderPlan,
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
pub fn export_animation_to_react(
    project_path: String,
    document: strut_core::Document,
    animation_name: String,
    output_dir: Option<String>,
) -> Result<ExportResult, String> {
    let root = ensure_project_root(&project_path)?;
    let default_name = sanitize_token(&animation_name);

    let export_dir = if let Some(dir) = output_dir.as_deref().map(str::trim).filter(|dir| !dir.is_empty()) {
        let requested = PathBuf::from(dir);
        if requested.is_absolute() {
            requested
        } else {
            root.join(requested)
        }
    } else {
        root.join("exports").join(format!("{default_name}-react"))
    };

    fs::create_dir_all(&export_dir).map_err(|error| {
        format!("Failed to create export directory: {}", error)
    })?;

    let files = react_export_files(&document);
    let mut exported_files = Vec::new();

    for (file_name, content) in files {
        let file_path = export_dir.join(&file_name);
        fs::write(&file_path, content).map_err(|error| {
            format!("Failed to write {}: {}", file_name.display(), error)
        })?;

        exported_files.push(ExportedFile {
            name: file_name.display().to_string(),
            path: file_path.display().to_string(),
        });
    }

    Ok(ExportResult {
        success: true,
        output_dir: export_dir.display().to_string(),
        files: exported_files,
    })
}

fn react_export_files(document: &strut_core::Document) -> Vec<(PathBuf, String)> {
    let scene_json = serde_json::to_string_pretty(document).expect("document serializes");
    let component_template = r#"import type { ReactNode } from "react";
import scene from "./scene.json";

type StrutNode = {{
  id: string;
  name: string;
  kind: string;
  role?: string;
  shape: {{ type: string; [key: string]: unknown }};
  style: {{ fill?: string | null; stroke?: string | null; stroke_width?: number; opacity?: number }};
  children?: StrutNode[];
}};

type StrutKeyframe = {{ time_ms: number; value: {{ type: string; value: number }}; easing?: string }};
type StrutTrack = {{ target: string; property: string; keyframes: StrutKeyframe[] }};
type StrutTimeline = {{ name: string; duration_ms: number; tracks: StrutTrack[] }};
type StrutTransition = {{ to: string; timeline: string }};
type StrutStateMachine = {{ transitions?: StrutTransition[] }};
type StrutScene = {{
  name: string;
  artboards: Array<{{ width: number; height: number; nodes: StrutNode[] }}>;
  timelines?: StrutTimeline[];
  state_machines?: StrutStateMachine[];
}};

const strutScene = scene as StrutScene;
const defaultTitle = __STRUT_TITLE_JSON__;

function paint(value: string | null | undefined) {{
  return value ?? "none";
}}

function cssIdent(value: string) {{
  return value.replace(/[^a-zA-Z0-9_-]/g, "-");
}}

function numericKeyframes(track: StrutTrack) {{
  return (track.keyframes ?? []).filter((keyframe) => keyframe.value?.type === "number");
}}

function valueAt(track: StrutTrack | undefined, time: number, fallback: number) {{
  const frames = track ? numericKeyframes(track).sort((a, b) => a.time_ms - b.time_ms) : [];
  if (!frames.length) return fallback;
  if (time <= frames[0].time_ms) return frames[0].value.value;
  const last = frames[frames.length - 1];
  if (time >= last.time_ms) return last.value.value;
  for (let index = 0; index < frames.length - 1; index += 1) {{
    const left = frames[index];
    const right = frames[index + 1];
    if (time >= left.time_ms && time <= right.time_ms) {{
      const span = Math.max(1, right.time_ms - left.time_ms);
      const progress = (time - left.time_ms) / span;
      return left.value.value + (right.value.value - left.value.value) * progress;
    }}
  }}
  return fallback;
}}

function groupTracks(timeline: StrutTimeline) {{
  const groups = new Map<string, StrutTrack[]>();
  for (const track of timeline.tracks ?? []) {{
    if (!numericKeyframes(track).length) continue;
    groups.set(track.target, [...(groups.get(track.target) ?? []), track]);
  }}
  return groups;
}}

function transformCss(tracks: StrutTrack[], timeline: StrutTimeline, target: string) {{
  const transformTracks = tracks.filter((track) => ["translation.x", "translation.y", "rotation", "scale.x", "scale.y"].includes(track.property));
  if (!transformTracks.length) return "";
  const times = Array.from(new Set([0, timeline.duration_ms, ...transformTracks.flatMap((track) => numericKeyframes(track).map((frame) => frame.time_ms))])).sort((a, b) => a - b);
  const frames = times
    .map((time) => {{
      const percent = Math.max(0, Math.min(100, (time / Math.max(1, timeline.duration_ms)) * 100));
      const tx = valueAt(transformTracks.find((track) => track.property === "translation.x"), time, 0);
      const ty = valueAt(transformTracks.find((track) => track.property === "translation.y"), time, 0);
      const rotate = valueAt(transformTracks.find((track) => track.property === "rotation"), time, 0);
      const sx = valueAt(transformTracks.find((track) => track.property === "scale.x"), time, 1);
      const sy = valueAt(transformTracks.find((track) => track.property === "scale.y"), time, 1);
      return `${{percent}}% {{ transform: translate(${{tx.toFixed(2)}}px, ${{ty.toFixed(2)}}px) rotate(${{rotate.toFixed(2)}}deg) scale(${{sx.toFixed(3)}}, ${{sy.toFixed(3)}}); }}`;
    }})
    .join("\n");
  return `@keyframes strut-${{cssIdent(timeline.name)}}-${{cssIdent(target)}}-transform {{\n${{frames}}\n}}\n`;
}}

function scalarCss(track: StrutTrack, timeline: StrutTimeline) {{
  if (track.property !== "opacity") return "";
  const frames = numericKeyframes(track)
    .sort((a, b) => a.time_ms - b.time_ms)
    .map((keyframe) => `${{Math.max(0, Math.min(100, (keyframe.time_ms / Math.max(1, timeline.duration_ms)) * 100))}}% {{ opacity: ${{keyframe.value.value.toFixed(3)}}; }}`)
    .join("\n");
  return `@keyframes strut-${{cssIdent(timeline.name)}}-${{cssIdent(track.target)}}-${{cssIdent(track.property)}} {{\n${{frames}}\n}}\n`;
}}

function activeTimelines(state: string, playAll: boolean) {{
  const timelines = strutScene.timelines ?? [];
  if (playAll) return timelines;
  const names = new Set<string>([state]);
  for (const machine of strutScene.state_machines ?? []) {{
    for (const transition of machine.transitions ?? []) {{
      if (transition.to === state) names.add(transition.timeline);
    }}
  }}
  return timelines.filter((timeline) => names.has(timeline.name) || timeline.name.startsWith(state));
}}

function animationCss(state: string, playAll: boolean) {{
  const rules: string[] = [];
  for (const timeline of activeTimelines(state, playAll)) {{
    for (const [target, tracks] of groupTracks(timeline)) {{
      const animations: string[] = [];
      const transformRule = transformCss(tracks, timeline, target);
      if (transformRule) {{
        rules.push(transformRule);
        animations.push(`strut-${{cssIdent(timeline.name)}}-${{cssIdent(target)}}-transform ${{timeline.duration_ms}}ms ease-in-out infinite`);
      }}
      for (const track of tracks) {{
        const scalarRule = scalarCss(track, timeline);
        if (scalarRule) {{
          rules.push(scalarRule);
          animations.push(`strut-${{cssIdent(timeline.name)}}-${{cssIdent(track.target)}}-${{cssIdent(track.property)}} ${{timeline.duration_ms}}ms ease-in-out infinite`);
        }}
      }}
      if (animations.length) {{
        rules.push(`[data-strut-id="${{target}}"] {{ transform-box: fill-box; transform-origin: center; animation: ${{animations.join(", ")}}; }}`);
      }}
    }}
  }}
  return rules.join("\n");
}}

function renderNode(node: StrutNode): ReactNode {{
  const style = node.style ?? {{}};
  const common = {{
    key: node.id,
    "data-strut-id": node.id,
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

export function StrutAnimation({{ state = "idle", title = defaultTitle, playAll = true }}: {{ state?: string; title?: string; playAll?: boolean }}) {{
  const artboard = strutScene.artboards[0];
  return (
    <svg viewBox={{`0 0 ${{artboard.width}} ${{artboard.height}}`}} role="img" aria-label={{title}} data-strut-state={{state}}>
      <style>{{animationCss(state, playAll)}}</style>
      {{artboard.nodes.map(renderNode)}}
    </svg>
  );
}}

export default StrutAnimation;
"#;
    let component = component_template
        .replace(
            "__STRUT_TITLE_JSON__",
            &serde_json::to_string(&document.name).expect("title serializes"),
        )
        .replace("{{", "{")
        .replace("}}", "}");
    let readme = format!(
        "# Strut React Export\n\nGenerated from `{}`.\n\n```tsx\nimport {{ StrutAnimation }} from \"./StrutAnimation\";\n\nexport function Example() {{\n  return <StrutAnimation state=\"idle\" playAll />;\n}}\n```\n\nThe component renders the validated `.strut` document as SVG and maps numeric Strut timeline tracks to CSS keyframe playback. Coding agents can edit `scene.json`, re-run `strut verify`, and keep the React wrapper unchanged.\n",
        document.name
    );
    vec![
        (PathBuf::from("scene.json"), scene_json),
        (PathBuf::from("StrutAnimation.tsx"), component),
        (PathBuf::from("README.md"), readme),
    ]
}
