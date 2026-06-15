use serde_json::{json, Value};
use std::path::PathBuf;
use crate::*;
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
    animations.retain(|animation| animation.id != record.id);
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

    let text = call_provider_for_assistant(&user_prompt, &provider, &references, &system_prompt).await?;

    let mut initial_result = match parse_assistant_result_from_text(&text) {
        Ok(result) => result,
        Err(first_error) => {
            if classify_request_intent(&user_prompt) == RequestIntent::Conversation {
                return Ok(AssistantResult::Chat {
                    message: text,
                    source: "raw".to_string(),
                });
            }

            let repair_prompt = generation_plan_repair_prompt(&user_prompt, &text, &first_error);
            let repair_text = match call_provider_for_assistant(&repair_prompt, &provider, &references, &system_prompt).await {
                Ok(t) => t,
                Err(e) => return Err(format!("Repair generation failed: {}", e)),
            };

            match parse_assistant_result_from_text(&repair_text) {
                Ok(result) => result,
                Err(repair_error) => {
                    let compact_prompt = compact_plan_prompt(&user_prompt, &repair_error);
                    let compact_text = match call_provider_for_assistant(&compact_prompt, &provider, &references, &system_prompt).await {
                        Ok(t) => t,
                        Err(e) => return Err(format!("Compact plan generation failed: {}", e)),
                    };

                    match parse_assistant_result_from_text(&compact_text) {
                        Ok(result) => result,
                        Err(plan_error) => {
                            return Err(format!(
                                "Model did not return a valid Strut document after 3 attempts.\nFirst error: {first_error}\nRepair error: {repair_error}\nPlan error: {plan_error}\nResponse preview: {}",
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
                let planned_doc = document_from_generation_plan_value(document_value)?;
                if kind == "document_created" {
                    return Ok(AssistantResult::DocumentCreated {
                        message,
                        source: "llm".to_string(),
                        document: planned_doc,
                    });
                } else {
                    return Ok(AssistantResult::DocumentUpdated {
                        message,
                        source: "llm".to_string(),
                        document: planned_doc,
                    });
                }
            }
            _ => return Err(format!("Unknown AssistantResult kind: {}", kind)),
        }
    }
    Err("Missing AssistantResult kind".to_string())
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