use crate::*;
use reqwest::header;
use serde_json::{json, Value};

fn openrouter_chat_url(endpoint: &str) -> String {
    let base = endpoint_base(endpoint);
    if base.ends_with("/chat/completions") { base } else { format!("{base}/chat/completions") }
}

async fn openrouter_text(config: &ByokProviderConfig, prompt: &str, system_prompt: &str, references: &[ReferenceImageInput]) -> Result<String, String> {
    ensure_byok_config(config)?;
    let client = http_client()?;
    let user_content = if references.is_empty() {
        json!(prompt)
    } else {
        let mut content = vec![json!({"type":"text","text": prompt_with_reference_context(prompt, references)})];
        content.extend(references.iter().map(|reference| json!({"type":"image_url","image_url":{"url":reference.data_url}})));
        json!(content)
    };
    let payload = json!({
        "model": config.model.trim(),
        "messages": [
            {"role":"system","content": system_prompt},
            {"role":"user","content": user_content}
        ],
        "temperature": 0.15,
        "stream": false,
        "max_tokens": 2048
    });
    let token = config.api_key.as_deref().unwrap_or_default();
    let auth_scheme = ["Bear", "er"].concat();
    let response = client.post(openrouter_chat_url(&config.endpoint))
        .header(header::ACCEPT, "application/json")
        .header(header::ACCEPT_ENCODING, "identity")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("{auth_scheme} {token}"))
        .header("HTTP-Referer", "https://github.com/amaansyed27/Strut")
        .header("X-Title", "Strut Studio")
        .header("X-OpenRouter-Metadata", "disabled")
        .json(&payload)
        .send()
        .await
        .map_err(|error| format!("OpenRouter request failed: {error}"))?;
    let status = response.status();
    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(error) if !status.is_success() => return Err(format!("OpenRouter HTTP {} (body unavailable: {error})", status.as_u16())),
        Err(error) => return Err(format!("OpenRouter body unavailable: {error}")),
    };
    let body = String::from_utf8_lossy(&bytes).to_string();
    if !status.is_success() { return Err(format!("OpenRouter {}", http_error_preview(status.as_u16(), &body))); }
    let json_body = serde_json::from_str::<Value>(&body).map_err(|error| format!("OpenRouter returned non-JSON: {error}. Body: {}", response_preview(&body)))?;
    json_body.pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("OpenRouter response missing message content. Body: {}", response_preview(&body)))
}

fn normalize_openrouter_value(mut value: Value) -> Value {
    let fallback_name = value.get("message").and_then(Value::as_str).unwrap_or("Generated animation").to_string();
    if let Some(plan) = value.get_mut("document").and_then(|document| document.get_mut("plan")) {
        if plan.get("name").and_then(Value::as_str).unwrap_or("").trim().is_empty() { plan["name"] = json!(fallback_name); }
        if plan.get("id").and_then(Value::as_str).unwrap_or("").trim().is_empty() { plan["id"] = json!("openrouter_generated_motion"); }
        if plan.get("subject").is_none() { plan["subject"] = json!({"classification":"object","label":"generated animation"}); }
    }
    value
}

fn parse_openrouter_result(text: &str) -> Result<AssistantResult, String> {
    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        let value = normalize_openrouter_value(value);
        if let Some(kind) = value.get("kind").and_then(Value::as_str) {
            let message = value.get("message").and_then(Value::as_str).unwrap_or("").to_string();
            if kind == "chat" { return Ok(AssistantResult::Chat { message, source: "openrouter".to_string() }); }
            if matches!(kind, "document_created" | "document_updated") {
                let document_value = value.get("document").ok_or_else(|| "OpenRouter result missing document".to_string())?;
                if let Ok(document) = serde_json::from_value::<strut_core::Document>(document_value.clone()) {
                    strut_format::validate_document(&document).map_err(|error| error.to_string())?;
                    return if kind == "document_created" {
                        Ok(AssistantResult::DocumentCreated { message, source: "openrouter".to_string(), document, plan_summary: None, operation_count: None })
                    } else {
                        Ok(AssistantResult::DocumentUpdated { message, source: "openrouter".to_string(), document, plan_summary: None, operation_count: None })
                    };
                }
            }
        }
        if let Ok(result) = crate::commands::parse_assistant_result_value(value) { return Ok(result); }
    }
    crate::commands::parse_assistant_result_from_text(text)
}

#[tauri::command]
pub async fn assistant_message_openrouter_v2(prompt: String, provider: Option<GenerationProvider>, references: Option<Vec<ReferenceImageInput>>, context: Option<GenerationContext>) -> Result<AssistantResult, String> {
    let references = references.unwrap_or_default();
    let provider = provider.ok_or_else(|| "Select OpenRouter first.".to_string())?;
    let config = provider.byok.as_ref().ok_or_else(|| "OpenRouter config missing.".to_string())?;
    let mut system_prompt = format!("{}\n\n{}", ASSISTANT_ROUTER_SYSTEM_PROMPT, DYNAMIC_GENERATION_SYSTEM_PROMPT);
    if let Some(ctx) = context.as_ref() {
        if let Some(project_name) = &ctx.project_name { system_prompt.push_str(&format!("\nProject: {project_name}")); }
        if let Some(chat_title) = &ctx.active_chat_title { system_prompt.push_str(&format!("\nChat: {chat_title}")); }
        if let Some(summary) = &ctx.current_document_summary { system_prompt.push_str(&format!("\n\nThe scene currently contains this document:\n{summary}")); }
    }
    if should_route_to_chat_response(&prompt, context.as_ref()) {
        let chat_prompt = chat_system_prompt(&prompt, context.as_ref());
        let message = openrouter_text(config, &prompt, &chat_prompt, &references).await?;
        return Ok(AssistantResult::Chat { message, source: "openrouter".to_string() });
    }
    let text = openrouter_text(config, &prompt, &system_prompt, &references).await?;
    let result = parse_openrouter_result(&text).map_err(|error| format!("OpenRouter returned invalid Strut JSON: {error}. Response preview: {}", response_preview(&text)))?;
    Ok(normalize_assistant_result_layout(result))
}
