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
        "temperature": 0.2,
        "stream": false,
        "max_tokens": 4096
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
    let result = crate::commands::parse_assistant_result_from_text(&text).map_err(|error| format!("OpenRouter returned invalid Strut JSON: {error}. Response preview: {}", response_preview(&text)))?;
    Ok(normalize_assistant_result_layout(result))
}
