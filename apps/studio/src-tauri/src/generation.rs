use serde_json::json;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::path::Path;
use std::fs;
use crate::*;




pub async fn byok_generate_text(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
) -> Result<String, String> {
    Ok(match config.provider_id.as_str() {
        "anthropic" => anthropic_message(prompt, config, references, system_prompt).await?,
        "gemini" => gemini_generate_content(prompt, config, references, system_prompt).await?,
        _ => openai_compatible_chat(prompt, config, references, system_prompt).await?,
    })
}

pub fn generate_document_with_sprite_python(prompt: &str) -> Result<strut_core::Document, String> {
    let package_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../packages/strut-python")
        .canonicalize()
        .map_err(|error| format!("sprite-python package was not found: {error}"))?;
    let example = sprite_python_example_for_prompt(prompt);
    let output = Command::new("python")
        .arg("-m")
        .arg("strut_python.cli")
        .arg(&example)
        .arg("--instruction")
        .arg(prompt)
        .arg("--json")
        .current_dir(&package_dir)
        .env("PYTHONPATH", package_dir.join("src"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("sprite-python failed to start: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "sprite-python exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    document_from_generation_plan_text(&stdout).map_err(|error| {
        format!("sprite-python emitted a plan that failed Rust validation: {error}")
    })
}

pub fn sprite_python_example_for_prompt(prompt: &str) -> String {
    let lower = prompt.to_lowercase();
    if lower.contains("logo") {
        "logo"
    } else if lower.contains("loader") || lower.contains("progress") || lower.contains("loading") {
        "loader"
    } else if lower.contains("mascot")
        || lower.contains("character")
        || lower.contains("duolingo")
        || lower.contains("codex pet")
    {
        "mascot"
    } else if lower.contains("icon") || lower.contains("badge") {
        "icon"
    } else if lower.contains("button") || lower.contains("microinteraction") || lower.contains("ui")
    {
        "ui"
    } else if lower.contains("dice") || lower.contains("die ") || lower.contains("rolling") {
        "dice"
    } else {
        "custom"
    }
    .to_string()
}

pub async fn chat_with_local_adapter(
    adapter_id: &str,
    prompt: &str,
    references: &[ReferenceImageInput],
    system_prompt: &str,
) -> Result<String, String> {
    let definition = local_adapter_definitions()
        .into_iter()
        .find(|definition| definition.id == adapter_id)
        .ok_or_else(|| format!("{adapter_id} is not registered"))?;

    if definition.generation == LocalGenerationKind::OllamaHttp {
        return chat_with_ollama(prompt, system_prompt).await;
    }

    if definition.generation == LocalGenerationKind::SpritePython {
        return Ok("I can help ideate motion and generate deterministic sprite-python plans locally. Ask for a specific asset, mascot, logo, UI state, icon, or animation when you want me to create a validated Strut scene.".to_string());
    }

    if definition.generation == LocalGenerationKind::AcpOnly {
        return Err(format!(
            "{} uses an ACP-style runtime. Strut detects it, but chat is disabled until ACP transport support lands.",
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
    let combined_prompt = format!("{}\n\n{}", system_prompt, prompt);
    let output = run_local_cli_command(
        &definition,
        &command,
        reference_dir,
        &combined_prompt,
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
    Ok(format!("{stdout}\n{stderr}").trim().to_string())
}

pub async fn generate_document_with_ollama(
    prompt: &str,
    references: &[ReferenceImageInput],
) -> Result<strut_core::Document, String> {
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

pub async fn openai_compatible_chat(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
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
                {"role": "system", "content": system_prompt.unwrap_or(GENERATION_PLAN_SYSTEM_PROMPT)},
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

pub async fn anthropic_message(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
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
            "system": system_prompt.unwrap_or(GENERATION_PLAN_SYSTEM_PROMPT),
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

pub async fn gemini_generate_content(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
) -> Result<String, String> {
    let client = http_client()?;
    let model = config.model.trim().trim_start_matches("models/");
    let mut parts = vec![json!({
        "text": format!("{}\nPrompt: {}", system_prompt.unwrap_or(GENERATION_PLAN_SYSTEM_PROMPT), prompt_with_reference_context(prompt, references))
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
pub async fn chat_with_ollama(prompt: &str, system_prompt: &str) -> Result<String, String> {
    let client = http_client()?;
    let response = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&json!({
            "model": "llama3.2",
            "system": system_prompt,
            "prompt": prompt,
            "stream": false
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let status = response.status();
    let body: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;
    if !status.is_success() {
        return Err(http_error_preview(status.as_u16(), &body.to_string()));
    }
    body.get("response")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Ollama response did not include a chat response".to_string())
}
