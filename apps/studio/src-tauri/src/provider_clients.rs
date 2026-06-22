use crate::*;
use reqwest::Response;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::Duration;

async fn json_or_detail(response: Response, label: &str) -> Result<Value, String> {
    let status = response.status();
    let bytes = response.bytes().await.map_err(|error| format!("{label} response body read failed: {error}"))?;
    let body = String::from_utf8_lossy(&bytes).to_string();
    if !status.is_success() {
        return Err(format!("{label} {}", http_error_preview(status.as_u16(), &body)));
    }
    serde_json::from_str::<Value>(&body)
        .map_err(|error| format!("{label} returned non-JSON: {error}. Body: {}", response_preview(&body)))
}

fn chat_url(endpoint: &str) -> String {
    let base = endpoint_base(endpoint);
    if base.ends_with("/chat/completions") { base } else { format!("{base}/chat/completions") }
}

fn wants_json(system_prompt: Option<&str>) -> bool {
    system_prompt
        .map(|prompt| {
            let lower = prompt.to_ascii_lowercase();
            lower.contains("json") || lower.contains("generationplan")
        })
        .unwrap_or(false)
}

fn supports_response_format(config: &ByokProviderConfig) -> bool {
    matches!(config.provider_id.as_str(), "openai" | "azure-openai")
}

async fn openai_like_text(prompt: &str, config: &ByokProviderConfig, references: &[ReferenceImageInput], system_prompt: Option<&str>, force_json: bool) -> Result<String, String> {
    let client = http_client()?;
    let user_content = if references.is_empty() {
        json!(prompt)
    } else {
        let mut content = vec![json!({"type":"text","text": prompt_with_reference_context(prompt, references)})];
        content.extend(references.iter().map(|reference| json!({"type":"image_url","image_url":{"url":reference.data_url}})));
        json!(content)
    };
    let mut payload = json!({
        "model": config.model.trim(),
        "messages": [
            {"role":"system","content": system_prompt.unwrap_or("You are Strut's AI assistant.")},
            {"role":"user","content": user_content}
        ],
        "temperature": 0.2
    });
    if force_json && supports_response_format(config) {
        payload["response_format"] = json!({"type":"json_object"});
    }
    let token = config.api_key.as_deref().unwrap_or_default();
    let mut request = client.post(chat_url(&config.endpoint)).bearer_auth(token).json(&payload);
    if config.provider_id == "openrouter" {
        request = request.header("HTTP-Referer", "https://github.com/amaansyed27/Strut").header("X-Title", "Strut Studio");
    }
    let body = json_or_detail(request.send().await.map_err(|error| error.to_string())?, provider_label(&config.provider_id)).await?;
    body.pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "provider response did not include choices[0].message.content".to_string())
}

async fn openai_like_text_resilient(prompt: &str, config: &ByokProviderConfig, references: &[ReferenceImageInput], system_prompt: Option<&str>) -> Result<String, String> {
    let force_json = wants_json(system_prompt);
    match openai_like_text(prompt, config, references, system_prompt, force_json).await {
        Ok(text) => Ok(text),
        Err(error) if force_json && should_retry_without_json_mode(&error) => openai_like_text(prompt, config, references, system_prompt, false).await,
        Err(error) => Err(error),
    }
}

fn should_retry_without_json_mode(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("response_format") || lower.contains("json_object") || lower.contains("unsupported") || lower.contains("http 400") || lower.contains("response body read failed") || lower.contains("decoding response body")
}

async fn anthropic_text(prompt: &str, config: &ByokProviderConfig, references: &[ReferenceImageInput], system_prompt: Option<&str>) -> Result<String, String> {
    let client = http_client()?;
    let mut content = vec![json!({"type":"text","text": prompt_with_reference_context(prompt, references)})];
    content.extend(references.iter().filter_map(|reference| {
        Some(json!({"type":"image","source":{"type":"base64","media_type":image_media_type(reference),"data":data_url_payload(&reference.data_url)?}}))
    }));
    let token = config.api_key.as_deref().unwrap_or_default();
    let response = client
        .post(format!("{}/v1/messages", endpoint_base(&config.endpoint)))
        .header("x-api-key", token)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": config.model.trim(),
            "max_tokens": 8192,
            "system": system_prompt.unwrap_or("You are Strut's AI assistant."),
            "messages": [{"role":"user","content": content}]
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let body = json_or_detail(response, "Anthropic").await?;
    let text = body.get("content").and_then(Value::as_array).map(|items| {
        items.iter().filter_map(|item| item.get("text").and_then(Value::as_str)).collect::<Vec<_>>().join("\n")
    }).unwrap_or_default();
    if text.trim().is_empty() { Err("Anthropic response did not include text content".to_string()) } else { Ok(text) }
}

async fn gemini_text(prompt: &str, config: &ByokProviderConfig, references: &[ReferenceImageInput], system_prompt: Option<&str>, model: &str) -> Result<String, String> {
    let client = http_client()?;
    let mut parts = vec![json!({"text": format!("{}\nPrompt: {}", system_prompt.unwrap_or("You are Strut's AI assistant."), prompt_with_reference_context(prompt, references))})];
    parts.extend(references.iter().filter_map(|reference| {
        Some(json!({"inline_data":{"mime_type":image_media_type(reference),"data":data_url_payload(&reference.data_url)?}}))
    }));
    let mut generation_config = json!({"temperature":0.2});
    if wants_json(system_prompt) { generation_config["responseMimeType"] = json!("application/json"); }
    let token = config.api_key.as_deref().unwrap_or_default();
    let response = client
        .post(format!("{}/v1beta/models/{}:generateContent?key={}", endpoint_base(&config.endpoint), model.trim().trim_start_matches("models/"), token))
        .json(&json!({"contents":[{"parts":parts}],"generationConfig":generation_config}))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let body = json_or_detail(response, "Gemini").await?;
    body.pointer("/candidates/0/content/parts/0/text")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "Gemini response did not include candidates[0].content.parts[0].text".to_string())
}

async fn gemini_text_resilient(prompt: &str, config: &ByokProviderConfig, references: &[ReferenceImageInput], system_prompt: Option<&str>) -> Result<String, String> {
    let model = config.model.trim().trim_start_matches("models/");
    match gemini_text(prompt, config, references, system_prompt, model).await {
        Ok(text) => Ok(text),
        Err(error) if model.starts_with("gemini-3") && (error.to_ascii_lowercase().contains("not found") || error.to_ascii_lowercase().contains("http 404")) => gemini_text(prompt, config, references, system_prompt, "gemini-2.5-flash").await,
        Err(error) => Err(error),
    }
}

pub async fn byok_generate_text_v2(prompt: &str, config: &ByokProviderConfig, references: &[ReferenceImageInput], system_prompt: Option<&str>) -> Result<String, String> {
    ensure_byok_config(config)?;
    match config.provider_id.as_str() {
        "anthropic" => anthropic_text(prompt, config, references, system_prompt).await,
        "gemini" => gemini_text_resilient(prompt, config, references, system_prompt).await,
        _ => openai_like_text_resilient(prompt, config, references, system_prompt).await,
    }
}

fn local_direct_args(definition: &LocalAdapterDefinition, reference_dir: Option<&Path>, prompt: &str) -> (Vec<String>, String) {
    match definition.id {
        "codex" => {
            let mut args = vec!["exec".to_string(), "--json".to_string(), "--skip-git-repo-check".to_string()];
            if let Some(dir) = reference_dir { args.extend(["--add-dir".to_string(), dir.display().to_string()]); }
            args.push(prompt.to_string());
            (args, String::new())
        }
        "claude-code" => (vec!["-p".to_string(), prompt.to_string(), "--output-format".to_string(), "stream-json".to_string()], String::new()),
        "opencode" => (vec!["run".to_string(), prompt.to_string(), "--format".to_string(), "json".to_string()], String::new()),
        "cursor-agent" => (vec!["--print".to_string(), prompt.to_string(), "--output-format".to_string(), "stream-json".to_string()], String::new()),
        "qoder" => (vec!["-p".to_string(), prompt.to_string(), "--output-format".to_string(), "stream-json".to_string()], String::new()),
        "copilot-cli" => (vec!["--output-format".to_string(), "json".to_string(), prompt.to_string()], String::new()),
        "gemini-cli" | "qwen" => (local_generation_args(definition, reference_dir), prompt.to_string()),
        _ => (local_generation_args(definition, reference_dir), prompt.to_string()),
    }
}

async fn local_adapter_text_v2(adapter_id: &str, prompt: &str, references: &[ReferenceImageInput], system_prompt: &str) -> Result<String, String> {
    let definition = local_adapter_definitions().into_iter().find(|definition| definition.id == adapter_id).ok_or_else(|| format!("{adapter_id} is not registered"))?;
    if definition.generation == LocalGenerationKind::OllamaHttp { return chat_with_ollama(prompt, system_prompt).await; }
    if definition.generation == LocalGenerationKind::SpritePython { return Ok("sprite-python provider is available for deterministic local recipe generation; interactive chat is not required for this engine.".to_string()); }
    if definition.generation == LocalGenerationKind::AcpOnly { return Err(format!("{} is detected, but ACP transport support is not implemented yet.", definition.name)); }

    let command = resolve_adapter_command(&definition).ok_or_else(|| format!("{} was not found on PATH or common tool directories", definition.commands.join(" / ")))?;
    let reference_files = write_reference_files(references)?;
    let reference_dir = reference_files.as_ref().map(|files| files.directory.as_path());
    let combined_prompt = format!("{}\n\n{}", system_prompt, prompt);
    let (args, stdin_text) = local_direct_args(&definition, reference_dir, &combined_prompt);
    let output = run_command_with_stdin(&command, &args, &local_generation_env(&definition), None, &stdin_text, Duration::from_secs(240));
    let _ = reference_files.as_ref().map(|files| fs::remove_dir_all(&files.directory));
    let output = output?;
    if !output.ok { return Err(command_output_preview(&output.stdout, &output.stderr)); }
    Ok(format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)).trim().to_string())
}

async fn call_provider_v2(prompt: &str, provider: &GenerationProvider, references: &[ReferenceImageInput], system_prompt: &str) -> Result<String, String> {
    match provider.mode.as_str() {
        "byok" => byok_generate_text_v2(prompt, provider.byok.as_ref().ok_or_else(|| "BYOK provider config missing".to_string())?, references, Some(system_prompt)).await,
        "local" => {
            let adapter_id = provider.local_adapter_id.as_ref().ok_or_else(|| "Select a local CLI or Ollama adapter".to_string())?;
            let raw = local_adapter_text_v2(adapter_id, prompt, references, system_prompt).await?;
            Ok(cli_assistant_text(&raw))
        }
        _ => Err("Unknown provider mode".to_string()),
    }
}

async fn improve_if_needed(prompt: &str, provider: &GenerationProvider, references: &[ReferenceImageInput], system_prompt: &str, result: AssistantResult) -> AssistantResult {
    let mut current = result;
    for _ in 0..2 {
        let Some(retry_prompt) = quality_repair_prompt(prompt, &current) else {
            return normalize_assistant_result_layout(current);
        };
        let Ok(retry_text) = call_provider_v2(&retry_prompt, provider, references, system_prompt).await else {
            return normalize_assistant_result_layout(current);
        };
        match crate::commands::parse_assistant_result_from_text(&retry_text) {
            Ok(retry_result) => current = retry_result,
            Err(_) => return normalize_assistant_result_layout(current),
        }
    }
    if quality_repair_prompt(prompt, &current).is_some() {
        return AssistantResult::Chat {
            source: "quality-gate".to_string(),
            message: "Generation was blocked because the provider kept returning an underbuilt animation that missed requested states, semantic layers, or active motion. Try again with the same prompt; Strut will now force a stricter repair prompt instead of saving another broken preview.".to_string(),
        };
    }
    normalize_assistant_result_layout(current)
}

#[tauri::command]
pub async fn assistant_message_v2(prompt: String, provider: Option<GenerationProvider>, references: Option<Vec<ReferenceImageInput>>, context: Option<GenerationContext>) -> Result<AssistantResult, String> {
    let references = references.unwrap_or_default();
    let provider = provider.ok_or_else(|| "Select a real local CLI, Ollama, or BYOK provider before generating.".to_string())?;
    let mut system_prompt = format!("{}\n\n{}", ASSISTANT_ROUTER_SYSTEM_PROMPT, DYNAMIC_GENERATION_SYSTEM_PROMPT);
    if let Some(ctx) = context.as_ref() {
        if let Some(project_name) = &ctx.project_name { system_prompt.push_str(&format!("\nProject: {project_name}")); }
        if let Some(chat_title) = &ctx.active_chat_title { system_prompt.push_str(&format!("\nChat: {chat_title}")); }
        if let Some(summary) = &ctx.current_document_summary { system_prompt.push_str(&format!("\n\nThe scene currently contains this document:\n{summary}")); }
    }
    if should_route_to_chat_response(&prompt, context.as_ref()) {
        let chat_prompt = chat_system_prompt(&prompt, context.as_ref());
        let message = call_provider_v2(&prompt, &provider, &references, &chat_prompt).await?;
        return Ok(AssistantResult::Chat { message, source: "llm".to_string() });
    }
    let text = call_provider_v2(&prompt, &provider, &references, &system_prompt).await?;
    match crate::commands::parse_assistant_result_from_text(&text) {
        Ok(result) => Ok(improve_if_needed(&prompt, &provider, &references, &system_prompt, result).await),
        Err(first_error) => {
            let repair_prompt = generation_plan_repair_prompt(&prompt, &text, &first_error);
            let repair_text = call_provider_v2(&repair_prompt, &provider, &references, &system_prompt).await?;
            match crate::commands::parse_assistant_result_from_text(&repair_text) {
                Ok(result) => Ok(improve_if_needed(&prompt, &provider, &references, &system_prompt, result).await),
                Err(repair_error) => {
                    let compact_prompt = compact_plan_prompt(&prompt, &repair_error);
                    let compact_text = call_provider_v2(&compact_prompt, &provider, &references, &system_prompt).await?;
                    match crate::commands::parse_assistant_result_from_text(&compact_text) {
                        Ok(result) => Ok(improve_if_needed(&prompt, &provider, &references, &system_prompt, result).await),
                        Err(plan_error) => Err(format!("Provider did not return valid Strut animation JSON after 3 attempts. First error: {first_error}. Repair error: {repair_error}. Plan error: {plan_error}. Response preview: {}", response_preview(&compact_text))),
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub async fn test_byok_provider_v2(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
    ensure_byok_config(&config)?;
    match byok_generate_text_v2("Return exactly: strut-provider-ok", &config, &[], Some("You are a provider smoke test. Return only the requested text.")).await {
        Ok(text) => Ok(ProviderOperationResult { ok: true, status: format!("{} ready", provider_label(&config.provider_id)), detail: format!("provider returned: {}", response_preview(&text)) }),
        Err(error) => Ok(ProviderOperationResult { ok: false, status: format!("{} failed smoke test", provider_label(&config.provider_id)), detail: error }),
    }
}

#[tauri::command]
pub async fn test_local_adapter_v2(adapter_id: String) -> ProviderOperationResult {
    match local_adapter_text_v2(&adapter_id, "Return exactly: strut-local-provider-ok", &[], "You are a local provider connection test. Return only the requested text and no markdown.").await {
        Ok(text) => ProviderOperationResult { ok: true, status: format!("{adapter_id} ready"), detail: format!("provider returned: {}", response_preview(&cli_assistant_text(&text))) },
        Err(error) => ProviderOperationResult { ok: false, status: format!("{adapter_id} failed smoke test"), detail: error },
    }
}

#[tauri::command]
pub fn save_byok_provider_v2(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
    ensure_byok_config(&config)?;
    let saved = SavedByokProviderConfig { provider_id: config.provider_id, endpoint: config.endpoint, model: config.model };
    let path = provider_config_path()?;
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|error| error.to_string())?; }
    fs::write(&path, serde_json::to_string_pretty(&saved).map_err(|error| error.to_string())?).map_err(|error| error.to_string())?;
    Ok(ProviderOperationResult { ok: true, status: "provider config saved".to_string(), detail: format!("saved endpoint and model to {}; settings stay local", path.display()) })
}
