use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use crate::*;

async fn response_json_or_detail(response: reqwest::Response, label: &str) -> Result<Value, String> {
    let status = response.status();
    let bytes = response.bytes().await.map_err(|error| format!("{label} response body read failed: {error}"))?;
    let body = String::from_utf8_lossy(&bytes).to_string();
    if !status.is_success() {
        return Err(format!("{label} {}", http_error_preview(status.as_u16(), &body)));
    }
    serde_json::from_str::<Value>(&body).map_err(|error| {
        format!("{label} returned non-JSON response: {error}. Body preview: {}", response_preview(&body))
    })
}

pub async fn byok_generate_text(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
) -> Result<String, String> {
    byok_generate_text_v2(prompt, config, references, system_prompt).await
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
    document_from_generation_plan_text(&stdout)
        .map_err(|error| format!("sprite-python emitted a plan that failed Rust validation: {error}"))
}

pub fn sprite_python_example_for_prompt(prompt: &str) -> String {
    let lower = prompt.to_lowercase();
    if lower.contains("logo") {
        "logo"
    } else if lower.contains("loader") || lower.contains("progress") || lower.contains("loading") {
        "loader"
    } else if lower.contains("mascot") || lower.contains("character") || lower.contains("duolingo") || lower.contains("codex pet") {
        "mascot"
    } else if lower.contains("icon") || lower.contains("badge") {
        "icon"
    } else if lower.contains("button") || lower.contains("microinteraction") || lower.contains("ui") {
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
        format!("{} was not found on PATH or common tool directories", definition.commands.join(" / "))
    })?;
    let reference_files = write_reference_files(references)?;
    let reference_dir = reference_files.as_ref().map(|files| files.directory.as_path());
    let combined_prompt = format!("{}\n\n{}", system_prompt, prompt);
    let output = run_local_cli_command(&definition, &command, reference_dir, &combined_prompt, Duration::from_secs(240))?;
    let _ = reference_files.as_ref().map(|files| fs::remove_dir_all(&files.directory));
    if !output.ok {
        return Err(command_output_preview(&output.stdout, &output.stderr));
    }
    Ok(format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)).trim().to_string())
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

    let body = response_json_or_detail(response, "Ollama").await?;
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
            let body = response_json_or_detail(repair_response, "Ollama repair").await?;
            let repair_text = body
                .get("response")
                .and_then(|value| value.as_str())
                .ok_or_else(|| "Ollama repair response did not include a response field".to_string())?;
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
                    let body = response_json_or_detail(plan_response, "Ollama compact plan").await?;
                    let plan_text = body
                        .get("response")
                        .and_then(|value| value.as_str())
                        .ok_or_else(|| "Ollama compact plan response did not include a response field".to_string())?;
                    document_from_generation_plan_text(plan_text).map_err(|plan_error| {
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
    byok_generate_text_v2(prompt, config, references, system_prompt).await
}

pub async fn anthropic_message(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
) -> Result<String, String> {
    byok_generate_text_v2(prompt, config, references, system_prompt).await
}

pub async fn gemini_generate_content(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
    system_prompt: Option<&str>,
) -> Result<String, String> {
    byok_generate_text_v2(prompt, config, references, system_prompt).await
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

    let body = response_json_or_detail(response, "Ollama chat").await?;
    body.get("response")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Ollama response did not include a chat response".to_string())
}
