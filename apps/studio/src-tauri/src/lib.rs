use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CHARACTER_SYSTEM_PROMPT: &str = "You convert design prompts into Strut character specs. Return only JSON with keys variant, name, accent, shell. variant must be one of floating-helper, scanner-bot, celebration-bot, owl-guide. accent and shell must be hex colors.";

#[derive(Debug, Clone, Serialize)]
struct AgentAdapterStatus {
    id: String,
    name: String,
    kind: String,
    command: Option<String>,
    installed: bool,
    detail: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ByokProviderConfig {
    provider_id: String,
    api_key: Option<String>,
    endpoint: String,
    model: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerationProvider {
    mode: String,
    local_adapter_id: Option<String>,
    byok: Option<ByokProviderConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReferenceImageInput {
    name: String,
    mime_type: String,
    data_url: String,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderOperationResult {
    ok: bool,
    status: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
struct GeneratedCharacter {
    document: strut_core::Document,
    source: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedByokProviderConfig {
    provider_id: String,
    endpoint: String,
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectInfo {
    name: String,
    path: String,
    files: Vec<ProjectFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectFile {
    name: String,
    path: String,
    kind: String,
}

#[tauri::command]
fn studio_status() -> strut_format::StudioStatus {
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
fn sample_document() -> strut_core::Document {
    let sample_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../samples/minimal-bot.strut");

    strut_format::read_strut_file(&sample_path)
        .map(|package| package.document)
        .unwrap_or_else(|_| strut_core::Document::sample_minimal_bot())
}

#[tauri::command]
fn default_project_location() -> String {
    default_projects_dir().display().to_string()
}

#[tauri::command]
fn create_project(name: String, location: String) -> Result<ProjectInfo, String> {
    let project_name = sanitize_project_name(&name)?;
    let root = if location.trim().is_empty() {
        default_projects_dir()
    } else {
        PathBuf::from(location.trim())
    };
    let project_path = root.join(&project_name);

    fs::create_dir_all(project_path.join("scenes")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join("assets")).map_err(|error| error.to_string())?;
    fs::create_dir_all(project_path.join("exports")).map_err(|error| error.to_string())?;

    let document = strut_core::Document::sample_minimal_bot();
    let document_json = serde_json::to_string_pretty(&document).map_err(|error| error.to_string())?;
    fs::write(project_path.join("scenes").join("starter.strut.json"), document_json)
        .map_err(|error| error.to_string())?;

    let metadata = json!({
        "name": project_name,
        "createdAt": unix_timestamp(),
        "format": "0.1.0",
        "mainScene": "scenes/starter.strut.json"
    });
    fs::write(
        project_path.join("strut.project.json"),
        serde_json::to_string_pretty(&metadata).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(project_info(project_name, project_path))
}

#[tauri::command]
async fn generate_character(
    prompt: String,
    provider: Option<GenerationProvider>,
    references: Option<Vec<ReferenceImageInput>>,
) -> Result<GeneratedCharacter, String> {
    let references = references.unwrap_or_default();
    if let Some(provider) = provider {
        match provider.mode.as_str() {
            "byok" => {
                let config = provider
                    .byok
                    .ok_or_else(|| "BYOK provider config missing".to_string())?;
                let spec = generate_spec_with_byok(&prompt, &config, &references).await?;
                let document = strut_core::Document::generate_character(spec);
                return Ok(GeneratedCharacter {
                    source: config.provider_id,
                    message: reference_message("Generated through BYOK provider", &references),
                    document,
                });
            }
            "local" if provider.local_adapter_id.as_deref() == Some("ollama") => {
                let spec = generate_spec_with_ollama(&prompt, &references).await?;
                let document = strut_core::Document::generate_character(spec);
                return Ok(GeneratedCharacter {
                    source: "ollama".to_string(),
                    message: reference_message("Generated through local Ollama", &references),
                    document,
                });
            }
            "local" => {
                let adapter = provider
                    .local_adapter_id
                    .unwrap_or_else(|| "local-agent".to_string());
                return Err(format!(
                    "{adapter} can be detected and tested, but Strut will not run a code-writing agent for generation until a command profile is configured"
                ));
            }
            _ => {}
        }
    }

    let prompt = prompt_with_reference_context(&prompt, &references);
    let spec = strut_core::character_spec_from_prompt(&prompt);
    let document = strut_core::Document::generate_character(spec);
    Ok(GeneratedCharacter {
        document,
        source: "built-in".to_string(),
        message: reference_message("Generated with built-in local generator", &references),
    })
}

#[tauri::command]
fn local_agent_adapters() -> Vec<AgentAdapterStatus> {
    local_adapter_definitions()
        .into_iter()
        .map(|(id, name, kind, command)| {
            let installed = command.is_some_and(command_available);
            let detail = match (command, installed) {
                (Some(command), true) => format!("{command} found on PATH"),
                (Some(command), false) => format!("{command} not found on PATH"),
                (None, _) => "configure local endpoint".to_string(),
            };

            AgentAdapterStatus {
                id: id.to_string(),
                name: name.to_string(),
                kind: kind.to_string(),
                command: command.map(str::to_string),
                installed,
                detail,
            }
        })
        .collect()
}

#[tauri::command]
fn test_local_adapter(adapter_id: String) -> ProviderOperationResult {
    let Some((_, name, _, command)) = local_adapter_definitions()
        .into_iter()
        .find(|(id, _, _, _)| *id == adapter_id)
    else {
        return ProviderOperationResult {
            ok: false,
            status: "unknown adapter".to_string(),
            detail: format!("{adapter_id} is not registered"),
        };
    };

    let Some(command) = command else {
        return ProviderOperationResult {
            ok: false,
            status: format!("{name} endpoint required"),
            detail: "This adapter uses an HTTP endpoint instead of a CLI binary".to_string(),
        };
    };

    if !command_available(command) {
        return ProviderOperationResult {
            ok: false,
            status: format!("{name} command missing"),
            detail: format!("{command} was not found on PATH"),
        };
    }

    match Command::new(command)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) if output.status.success() => ProviderOperationResult {
            ok: true,
            status: format!("{name} ready"),
            detail: command_output_preview(&output.stdout, &output.stderr),
        },
        Ok(output) => ProviderOperationResult {
            ok: false,
            status: format!("{name} returned an error"),
            detail: command_output_preview(&output.stdout, &output.stderr),
        },
        Err(error) => ProviderOperationResult {
            ok: false,
            status: format!("{name} failed to run"),
            detail: error.to_string(),
        },
    }
}

#[tauri::command]
async fn test_byok_provider(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
    ensure_byok_config(&config)?;
    let client = http_client()?;
    let result = match config.provider_id.as_str() {
        "anthropic" => {
            client
                .get(format!("{}/v1/models", endpoint_base(&config.endpoint)))
                .header("x-api-key", config.api_key.as_deref().unwrap_or_default())
                .header("anthropic-version", "2023-06-01")
                .send()
                .await
        }
        "gemini" => {
            client
                .get(format!(
                    "{}/v1beta/models?key={}",
                    endpoint_base(&config.endpoint),
                    config.api_key.as_deref().unwrap_or_default()
                ))
                .send()
                .await
        }
        _ => {
            client
                .get(format!("{}/models", endpoint_base(&config.endpoint)))
                .bearer_auth(config.api_key.as_deref().unwrap_or_default())
                .send()
                .await
        }
    }
    .map_err(|error| error.to_string())?;

    let status = result.status();
    let text = result.text().await.unwrap_or_default();
    if status.is_success() {
        Ok(ProviderOperationResult {
            ok: true,
            status: format!("{} ready", provider_label(&config.provider_id)),
            detail: "provider accepted the configured credentials".to_string(),
        })
    } else {
        Ok(ProviderOperationResult {
            ok: false,
            status: format!(
                "{} rejected credentials",
                provider_label(&config.provider_id)
            ),
            detail: http_error_preview(status.as_u16(), &text),
        })
    }
}

#[tauri::command]
fn save_byok_provider(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
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

fn local_adapter_definitions() -> Vec<(
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
)> {
    vec![
        ("ollama", "Ollama", "local-model", Some("ollama")),
        ("codex", "Codex", "local-agent", Some("codex")),
        ("gemini-cli", "Gemini CLI", "local-agent", Some("gemini")),
        ("claude-code", "Claude Code", "local-agent", Some("claude")),
        ("copilot-cli", "Copilot CLI", "local-agent", Some("gh")),
        (
            "antigravity",
            "Antigravity",
            "local-agent",
            Some("antigravity"),
        ),
        ("kiro", "Kiro", "local-agent", Some("kiro")),
        ("lm-studio", "LM Studio", "local-model", None),
    ]
}

fn default_projects_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        return PathBuf::from(home).join("Documents").join("Strut Projects");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("Strut Projects")
}

fn sanitize_project_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("project name is required".to_string());
    }

    let sanitized: String = trimmed
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_'))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if sanitized.is_empty() {
        Err("project name needs letters or numbers".to_string())
    } else {
        Ok(sanitized)
    }
}

fn project_info(name: String, path: PathBuf) -> ProjectInfo {
    ProjectInfo {
        name,
        path: path.display().to_string(),
        files: vec![
            ProjectFile {
                name: "strut.project.json".to_string(),
                path: path.join("strut.project.json").display().to_string(),
                kind: "project".to_string(),
            },
            ProjectFile {
                name: "starter.strut.json".to_string(),
                path: path
                    .join("scenes")
                    .join("starter.strut.json")
                    .display()
                    .to_string(),
                kind: "scene".to_string(),
            },
            ProjectFile {
                name: "assets".to_string(),
                path: path.join("assets").display().to_string(),
                kind: "folder".to_string(),
            },
            ProjectFile {
                name: "exports".to_string(),
                path: path.join("exports").display().to_string(),
                kind: "folder".to_string(),
            },
        ],
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn command_available(command: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };

    let candidates = command_candidates(command);
    std::env::split_paths(&paths).any(|path| {
        candidates
            .iter()
            .map(|candidate| path.join(candidate))
            .any(|candidate| candidate.is_file())
    })
}

fn command_candidates(command: &str) -> Vec<PathBuf> {
    if cfg!(windows) {
        let extensions = std::env::var_os("PATHEXT")
            .map(|value| {
                value
                    .to_string_lossy()
                    .split(';')
                    .filter(|extension| !extension.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .filter(|extensions| !extensions.is_empty())
            .unwrap_or_else(|| vec![".EXE".to_string(), ".CMD".to_string(), ".BAT".to_string()]);

        extensions
            .into_iter()
            .flat_map(|extension| {
                [
                    PathBuf::from(command),
                    PathBuf::from(format!("{command}{extension}")),
                    PathBuf::from(format!("{command}{}", extension.to_lowercase())),
                ]
            })
            .collect()
    } else {
        vec![PathBuf::from(command)]
    }
}

fn command_output_preview(stdout: &[u8], stderr: &[u8]) -> String {
    let mut text = String::from_utf8_lossy(stdout).trim().to_string();
    if text.is_empty() {
        text = String::from_utf8_lossy(stderr).trim().to_string();
    }
    if text.is_empty() {
        "command completed without output".to_string()
    } else {
        text.chars().take(220).collect()
    }
}

fn provider_config_path() -> Result<PathBuf, String> {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return Ok(PathBuf::from(appdata)
            .join("Strut")
            .join("providers")
            .join("byok.json"));
    }

    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home)
            .join("strut")
            .join("providers")
            .join("byok.json"));
    }

    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home)
            .join(".config")
            .join("strut")
            .join("providers")
            .join("byok.json"));
    }

    Err("could not resolve a local config directory".to_string())
}

fn ensure_byok_config(config: &ByokProviderConfig) -> Result<(), String> {
    if config
        .api_key
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        return Err(format!(
            "{} API key is required",
            provider_label(&config.provider_id)
        ));
    }
    if config.endpoint.trim().is_empty() {
        return Err("provider endpoint is required".to_string());
    }
    if config.model.trim().is_empty() {
        return Err("provider model is required".to_string());
    }
    Ok(())
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| error.to_string())
}

fn endpoint_base(endpoint: &str) -> String {
    endpoint.trim().trim_end_matches('/').to_string()
}

fn provider_label(provider_id: &str) -> &'static str {
    match provider_id {
        "openai" => "OpenAI",
        "anthropic" => "Anthropic",
        "gemini" => "Gemini",
        "openrouter" => "OpenRouter",
        "azure-openai" => "Azure OpenAI",
        "openai-compatible" => "OpenAI Compatible",
        _ => "Provider",
    }
}

fn http_error_preview(status: u16, body: &str) -> String {
    let preview: String = body.chars().take(260).collect();
    if preview.trim().is_empty() {
        format!("HTTP {status}")
    } else {
        format!("HTTP {status}: {preview}")
    }
}

fn data_url_payload(data_url: &str) -> Option<&str> {
    data_url.split_once(',').map(|(_, payload)| payload)
}

fn image_media_type(reference: &ReferenceImageInput) -> &str {
    if reference.mime_type.trim().is_empty() {
        return "image/png";
    }
    reference.mime_type.trim()
}

fn prompt_with_reference_context(prompt: &str, references: &[ReferenceImageInput]) -> String {
    if references.is_empty() {
        return prompt.to_string();
    }
    let names = references
        .iter()
        .map(|reference| {
            format!(
                "{} ({})",
                reference.name.trim(),
                image_media_type(reference)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{prompt}\n\nReference images attached: {names}. Inspect the image composition, silhouette, pose, palette, and visible parts, then return an editable Strut character spec that matches the reference direction."
    )
}

fn reference_message(base: &str, references: &[ReferenceImageInput]) -> String {
    if references.is_empty() {
        base.to_string()
    } else {
        format!(
            "{base} using {} reference image{}",
            references.len(),
            if references.len() == 1 { "" } else { "s" }
        )
    }
}

async fn generate_spec_with_byok(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<strut_core::CharacterSpec, String> {
    ensure_byok_config(config)?;
    let response_text = match config.provider_id.as_str() {
        "anthropic" => anthropic_message(prompt, config, references).await?,
        "gemini" => gemini_generate_content(prompt, config, references).await?,
        _ => openai_compatible_chat(prompt, config, references).await?,
    };

    parse_character_spec(&response_text)
}

async fn generate_spec_with_ollama(
    prompt: &str,
    references: &[ReferenceImageInput],
) -> Result<strut_core::CharacterSpec, String> {
    let client = http_client()?;
    let images = references
        .iter()
        .filter_map(|reference| data_url_payload(&reference.data_url).map(str::to_string))
        .collect::<Vec<_>>();
    let response = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&json!({
            "model": "llama3.2",
            "prompt": format!("{CHARACTER_SYSTEM_PROMPT}\nPrompt: {}", prompt_with_reference_context(prompt, references)),
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
    parse_character_spec(text)
}

async fn openai_compatible_chat(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {
    let client = http_client()?;
    let user_content = if references.is_empty() {
        json!(prompt)
    } else {
        let mut content = vec![json!({"type": "text", "text": prompt_with_reference_context(prompt, references)})];
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
                {"role": "system", "content": CHARACTER_SYSTEM_PROMPT},
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

async fn anthropic_message(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {
    let client = http_client()?;
    let mut content = vec![json!({"type": "text", "text": prompt_with_reference_context(prompt, references)})];
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
            "max_tokens": 512,
            "system": CHARACTER_SYSTEM_PROMPT,
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

async fn gemini_generate_content(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<String, String> {
    let client = http_client()?;
    let model = config.model.trim().trim_start_matches("models/");
    let mut parts = vec![json!({
        "text": format!("{CHARACTER_SYSTEM_PROMPT}\nPrompt: {}", prompt_with_reference_context(prompt, references))
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

fn parse_character_spec(text: &str) -> Result<strut_core::CharacterSpec, String> {
    let json_text = extract_json_object(text).unwrap_or(text);
    serde_json::from_str::<strut_core::CharacterSpec>(json_text)
        .map_err(|error| format!("model did not return a valid Strut character spec: {error}"))
}

fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    (end > start).then_some(&text[start..=end])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            studio_status,
            sample_document,
            default_project_location,
            create_project,
            generate_character,
            local_agent_adapters,
            test_local_adapter,
            test_byok_provider,
            save_byok_provider
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_agent_catalog_includes_requested_providers() {
        let adapters = local_agent_adapters();
        let ids = adapters
            .iter()
            .map(|adapter| adapter.id.as_str())
            .collect::<Vec<_>>();

        assert!(ids.contains(&"codex"));
        assert!(ids.contains(&"gemini-cli"));
        assert!(ids.contains(&"claude-code"));
        assert!(ids.contains(&"copilot-cli"));
        assert!(ids.contains(&"ollama"));
    }

    #[test]
    fn parses_json_spec_from_model_text() {
        let spec = parse_character_spec(
            r##"Here is JSON: {"variant":"owl-guide","name":"Owl Mascot","accent":"#78d64b","shell":"#8ee15a"}"##,
        )
        .expect("spec should parse");

        assert_eq!(spec.variant, "owl-guide");
        assert_eq!(spec.name.as_deref(), Some("Owl Mascot"));
    }

    #[test]
    fn provider_config_path_is_local() {
        let path = provider_config_path().expect("config path");
        assert!(path.ends_with("byok.json"));
    }

    #[test]
    fn project_name_is_sanitized() {
        assert_eq!(
            sanitize_project_name("  My Bot / Demo!! ").expect("project name"),
            "My Bot Demo"
        );
    }
}
