use crate::*;
use serde_json::{json, Value};

const OPENROUTER_COMPACT_PLAN_SYSTEM: &str = r#"You are Strut's compact animation planner.
Output only raw JSON. No markdown. No explanation. No full StrutDocument.
Return exactly this envelope:
{"plan":{"id":"short_id","name":"Human Name","subject":{"classification":"object|scene|ui|mascot|abstract","label":"subject"},"parts":[Part],"states":["idle","active","settle"],"timelines":[Timeline]},"operations":[]}
Part fields: id, name, role, geometry, style, motion_roles, constraints.
Geometry kinds only: rect, ellipse, path, text.
Style fields: fill, stroke, stroke_width, opacity.
Constraints must use allowed_properties: translation.x, translation.y, rotation, scale, scale.x, scale.y, opacity.
Timeline fields: id, name, duration_ms, loops, tracks.
Track fields: target, property, keyframes. Every target must match a part id.
Keyframe fields: time_ms, value as a number, easing.
Quality rules: build the exact subject from semantic editable layers; use 10-18 parts for dynamic objects; include shadow/depth/highlight/detail layers when the prompt asks for premium, reflective, 2.5D, or 3D-style motion; timelines must include active motion, not just opacity.
"#;

async fn openrouter_text(config: &ByokProviderConfig, prompt: &str, system_prompt: &str, references: &[ReferenceImageInput]) -> Result<String, String> {
    ensure_byok_config(config)?;
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
    let json_body = post_openrouter_chat_with_curl(&config.endpoint, token, &payload)?;
    json_body.pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("OpenRouter response missing message content. Body: {}", response_preview(&json_body.to_string())))
}

fn openrouter_generation_system_prompt(context: Option<&GenerationContext>) -> String {
    let mut text = OPENROUTER_COMPACT_PLAN_SYSTEM.to_string();
    if let Some(ctx) = context {
        text.push_str("\nWorkspace context:");
        if let Some(project_name) = &ctx.project_name { text.push_str(&format!("\n- Project: {project_name}")); }
        if let Some(chat_title) = &ctx.active_chat_title { text.push_str(&format!("\n- Chat: {chat_title}")); }
        if let Some(summary) = &ctx.current_document_summary { text.push_str(&format!("\n- Current scene: {summary}")); }
    }
    text
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
    if let Ok(document) = document_from_generation_plan_text(text) {
        return Ok(AssistantResult::DocumentCreated {
            message: "Generated compact OpenRouter animation plan.".to_string(),
            source: "openrouter-compact-plan".to_string(),
            document,
            plan_summary: None,
            operation_count: None,
        });
    }
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
    if should_route_to_chat_response(&prompt, context.as_ref()) {
        let chat_prompt = chat_system_prompt(&prompt, context.as_ref());
        let message = openrouter_text(config, &prompt, &chat_prompt, &references).await?;
        return Ok(AssistantResult::Chat { message, source: "openrouter".to_string() });
    }
    let system_prompt = openrouter_generation_system_prompt(context.as_ref());
    let text = openrouter_text(config, &prompt, &system_prompt, &references).await?;
    let result = parse_openrouter_result(&text).map_err(|error| format!("OpenRouter returned invalid compact Strut plan: {error}. Response preview: {}", response_preview(&text)))?;
    Ok(normalize_assistant_result_layout(result))
}
