use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CHARACTER_SYSTEM_PROMPT: &str = "You convert design prompts into Strut character specs. Return only JSON with keys variant, name, accent, shell. variant must be one of floating-helper, scanner-bot, celebration-bot, owl-guide. accent and shell must be hex colors.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalGenerationKind {
    OllamaHttp,
    StdinPrompt,
    AcpOnly,
}

#[derive(Debug, Clone)]
struct LocalAdapterDefinition {
    id: &'static str,
    name: &'static str,
    kind: &'static str,
    commands: &'static [&'static str],
    version_args: &'static [&'static str],
    generation: LocalGenerationKind,
}

#[derive(Debug)]
struct CommandRun {
    ok: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

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

#[derive(Debug)]
struct WrittenReferenceFiles {
    directory: PathBuf,
    paths: Vec<PathBuf>,
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

    let document = strut_core::Document::empty_scene(&project_name);
    let document_json =
        serde_json::to_string_pretty(&document).map_err(|error| error.to_string())?;
    fs::write(
        project_path.join("scenes").join("starter.strut.json"),
        document_json,
    )
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
    let provider = provider.ok_or_else(|| {
        "Select a real local CLI, Ollama, or BYOK provider before generating. Built-in fallback generation has been removed.".to_string()
    })?;

    match provider.mode.as_str() {
        "byok" => {
            let config = provider
                .byok
                .ok_or_else(|| "BYOK provider config missing".to_string())?;
            let spec = generate_spec_with_byok(&prompt, &config, &references).await?;
            let document = strut_core::Document::generate_character(spec);
            Ok(GeneratedCharacter {
                source: config.provider_id,
                message: reference_message("Generated through BYOK provider", &references),
                document,
            })
        }
        "local" => {
            let adapter_id = provider
                .local_adapter_id
                .ok_or_else(|| "Select a local CLI or Ollama adapter".to_string())?;
            let spec = generate_spec_with_local_adapter(&adapter_id, &prompt, &references).await?;
            let document = strut_core::Document::generate_character(spec);
            Ok(GeneratedCharacter {
                source: adapter_id,
                message: reference_message("Generated through local provider", &references),
                document,
            })
        }
        _ => Err("Unknown generation provider mode".to_string()),
    }
}

#[tauri::command]
fn local_agent_adapters() -> Vec<AgentAdapterStatus> {
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
fn test_local_adapter(adapter_id: String) -> ProviderOperationResult {
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
        let prompt = format!(
            "{CHARACTER_SYSTEM_PROMPT}\n\nReply with only this JSON: {{\"variant\":\"floating-helper\",\"name\":\"Smoke Bot\",\"accent\":\"#51bfd0\",\"shell\":\"#f6f1e8\"}}"
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
async fn test_byok_provider(config: ByokProviderConfig) -> Result<ProviderOperationResult, String> {
    ensure_byok_config(&config)?;
    let smoke_prompt = "Create a small floating helper character named Smoke Bot with a cyan accent. Return the Strut character spec JSON.";
    match generate_spec_with_byok(smoke_prompt, &config, &[]).await {
        Ok(_) => Ok(ProviderOperationResult {
            ok: true,
            status: format!("{} ready", provider_label(&config.provider_id)),
            detail: "provider completed a real structured generation smoke test".to_string(),
        }),
        Err(error) => Ok(ProviderOperationResult {
            ok: false,
            status: format!("{} failed smoke test", provider_label(&config.provider_id)),
            detail: error,
        }),
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

fn local_adapter_definitions() -> Vec<LocalAdapterDefinition> {
    vec![
        LocalAdapterDefinition {
            id: "ollama",
            name: "Ollama",
            kind: "local-model",
            commands: &["ollama"],
            version_args: &["--version"],
            generation: LocalGenerationKind::OllamaHttp,
        },
        LocalAdapterDefinition {
            id: "codex",
            name: "Codex",
            kind: "local-agent",
            commands: &["codex"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "gemini-cli",
            name: "Gemini CLI",
            kind: "local-agent",
            commands: &["gemini"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "claude-code",
            name: "Claude Code",
            kind: "local-agent",
            commands: &["claude", "openclaude"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "opencode",
            name: "OpenCode",
            kind: "local-agent",
            commands: &["opencode-cli", "opencode"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "cursor-agent",
            name: "Cursor Agent",
            kind: "local-agent",
            commands: &["cursor-agent"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "qwen",
            name: "Qwen Code",
            kind: "local-agent",
            commands: &["qwen"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "qoder",
            name: "Qoder CLI",
            kind: "local-agent",
            commands: &["qodercli", "qoder"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "copilot-cli",
            name: "Copilot CLI",
            kind: "local-agent",
            commands: &["copilot"],
            version_args: &["--version"],
            generation: LocalGenerationKind::StdinPrompt,
        },
        LocalAdapterDefinition {
            id: "antigravity",
            name: "Antigravity",
            kind: "local-agent",
            commands: &["antigravity"],
            version_args: &["--version"],
            generation: LocalGenerationKind::AcpOnly,
        },
        LocalAdapterDefinition {
            id: "kiro",
            name: "Kiro",
            kind: "local-agent",
            commands: &["kiro-cli", "kiro"],
            version_args: &["--version"],
            generation: LocalGenerationKind::AcpOnly,
        },
        LocalAdapterDefinition {
            id: "lm-studio",
            name: "LM Studio",
            kind: "local-model",
            commands: &["lms"],
            version_args: &["--version"],
            generation: LocalGenerationKind::AcpOnly,
        },
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
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_')
        })
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

fn resolve_adapter_command(definition: &LocalAdapterDefinition) -> Option<PathBuf> {
    definition
        .commands
        .iter()
        .find_map(|command| resolve_command(command))
}

fn resolve_command(command: &str) -> Option<PathBuf> {
    let candidates = command_candidates(command);
    command_search_dirs().into_iter().find_map(|path| {
        candidates
            .iter()
            .map(|candidate| path.join(candidate))
            .find(|candidate| candidate.is_file())
    })
}

fn command_search_dirs() -> Vec<PathBuf> {
    let mut dirs = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .unwrap_or_default();

    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let home = PathBuf::from(home);
        dirs.extend([
            home.join(".codex").join("bin"),
            home.join(".local").join("bin"),
            home.join(".npm-global").join("bin"),
            home.join("AppData").join("Roaming").join("npm"),
            home.join("AppData")
                .join("Local")
                .join("Programs")
                .join("cursor")
                .join("resources")
                .join("app")
                .join("bin"),
        ]);
    }

    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let program_files = PathBuf::from(program_files);
        dirs.extend([
            program_files.join("GitHub CLI"),
            program_files.join("Ollama"),
            program_files.join("Claude"),
        ]);
    }

    dirs
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

fn run_ollama_smoke_test(command: &Path) -> Result<CommandRun, String> {
    run_command_with_stdin(
        command,
        &["run".to_string(), "llama3.2".to_string()],
        &[],
        None,
        "Reply with only this JSON: {\"variant\":\"floating-helper\",\"name\":\"Smoke Bot\",\"accent\":\"#51bfd0\",\"shell\":\"#f6f1e8\"}",
        Duration::from_secs(60),
    )
}

fn run_local_cli_command(
    definition: &LocalAdapterDefinition,
    command: &Path,
    reference_dir: Option<&Path>,
    input: &str,
    timeout: Duration,
) -> Result<CommandRun, String> {
    let args = local_generation_args(definition, reference_dir);
    let env = local_generation_env(definition);
    run_command_with_stdin(command, &args, &env, None, input, timeout)
}

fn run_command_with_stdin(
    command: &Path,
    args: &[String],
    env: &[(&str, &str)],
    cwd: Option<&Path>,
    input: &str,
    timeout: Duration,
) -> Result<CommandRun, String> {
    let mut process = Command::new(command);
    process
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = cwd {
        process.current_dir(cwd);
    }
    for (key, value) in env {
        process.env(key, value);
    }

    let mut child = process.spawn().map_err(|error| error.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|error| error.to_string())?;
    }

    let started = Instant::now();
    loop {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            let output = child
                .wait_with_output()
                .map_err(|error| error.to_string())?;
            return Ok(CommandRun {
                ok: output.status.success(),
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }

        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "{} timed out after {} seconds",
                command.display(),
                timeout.as_secs()
            ));
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

fn local_generation_args(
    definition: &LocalAdapterDefinition,
    reference_dir: Option<&Path>,
) -> Vec<String> {
    match definition.id {
        "codex" => {
            let mut args = vec![
                "exec".to_string(),
                "--json".to_string(),
                "--skip-git-repo-check".to_string(),
            ];
            if cfg!(windows) {
                args.extend(["--sandbox".to_string(), "danger-full-access".to_string()]);
            } else {
                args.extend([
                    "--sandbox".to_string(),
                    "workspace-write".to_string(),
                    "-c".to_string(),
                    "sandbox_workspace_write.network_access=true".to_string(),
                ]);
            }
            if let Some(dir) = reference_dir {
                args.extend(["--add-dir".to_string(), dir.display().to_string()]);
            }
            args
        }
        "gemini-cli" => vec![
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--yolo".to_string(),
        ],
        "claude-code" => vec![
            "-p".to_string(),
            "--input-format".to_string(),
            "text".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
            "--permission-mode".to_string(),
            "bypassPermissions".to_string(),
        ],
        "opencode" => vec![
            "run".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--dangerously-skip-permissions".to_string(),
        ],
        "cursor-agent" => vec![
            "--print".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--stream-partial-output".to_string(),
            "--force".to_string(),
            "--trust".to_string(),
        ],
        "qwen" => vec!["--yolo".to_string()],
        "qoder" => vec![
            "-p".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--yolo".to_string(),
        ],
        "copilot-cli" => vec![
            "--allow-all-tools".to_string(),
            "--output-format".to_string(),
            "json".to_string(),
        ],
        _ => definition
            .version_args
            .iter()
            .map(|arg| arg.to_string())
            .collect(),
    }
}

fn local_generation_env(definition: &LocalAdapterDefinition) -> Vec<(&'static str, &'static str)> {
    match definition.id {
        "gemini-cli" => vec![("GEMINI_CLI_TRUST_WORKSPACE", "true")],
        _ => Vec::new(),
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
    ensure_safe_endpoint(&config.endpoint)?;
    if config.model.trim().is_empty() {
        return Err("provider model is required".to_string());
    }
    Ok(())
}

fn ensure_safe_endpoint(endpoint: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(endpoint.trim())
        .map_err(|error| format!("provider endpoint is not a valid URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("provider endpoint must use http or https".to_string());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "provider endpoint must include a host".to_string())?;
    if host.eq_ignore_ascii_case("localhost") {
        return Ok(());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if ip.is_loopback() {
            return Ok(());
        }
        if is_blocked_internal_ip(ip) {
            return Err("provider endpoint cannot target private, link-local, or unspecified network addresses unless it is loopback".to_string());
        }
    }
    Ok(())
}

fn is_blocked_internal_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private() || ip.is_link_local() || ip.is_unspecified() || ip.is_broadcast()
        }
        IpAddr::V6(ip) => ip.is_unique_local() || ip.is_unicast_link_local() || ip.is_unspecified(),
    }
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

fn local_character_prompt(
    prompt: &str,
    references: &[ReferenceImageInput],
    reference_files: Option<&WrittenReferenceFiles>,
) -> String {
    let mut text = format!(
        "{CHARACTER_SYSTEM_PROMPT}\n\nUser request:\n{}",
        prompt_with_reference_context(prompt, references)
    );
    if let Some(files) = reference_files {
        if !files.paths.is_empty() {
            text.push_str("\n\nReference files written for this run:");
            for path in &files.paths {
                text.push_str(&format!("\n- {}", path.display()));
            }
            text.push_str("\nUse those image files as visual references when your runtime supports local file inspection.");
        }
    }
    text.push_str("\n\nReturn only the JSON object. Do not include markdown.");
    text
}

fn write_reference_files(
    references: &[ReferenceImageInput],
) -> Result<Option<WrittenReferenceFiles>, String> {
    if references.is_empty() {
        return Ok(None);
    }

    let directory = std::env::temp_dir().join(format!("strut-references-{}", unix_timestamp()));
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let mut paths = Vec::new();
    for (index, reference) in references.iter().enumerate() {
        let Some(payload) = data_url_payload(&reference.data_url) else {
            continue;
        };
        let bytes = general_purpose::STANDARD
            .decode(payload)
            .map_err(|error| format!("could not decode {}: {error}", reference.name))?;
        let extension = image_extension(reference);
        let name = sanitize_file_stem(&reference.name);
        let path = directory.join(format!("{index:02}-{name}.{extension}"));
        fs::write(&path, bytes).map_err(|error| error.to_string())?;
        paths.push(path);
    }

    Ok(Some(WrittenReferenceFiles { directory, paths }))
}

fn sanitize_file_stem(name: &str) -> String {
    let sanitized = name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .collect::<String>();
    if sanitized.is_empty() {
        "reference".to_string()
    } else {
        sanitized
    }
}

fn image_extension(reference: &ReferenceImageInput) -> &'static str {
    match image_media_type(reference) {
        "image/jpeg" => "jpg",
        "image/svg+xml" => "svg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "png",
    }
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

async fn generate_spec_with_local_adapter(
    adapter_id: &str,
    prompt: &str,
    references: &[ReferenceImageInput],
) -> Result<strut_core::CharacterSpec, String> {
    let definition = local_adapter_definitions()
        .into_iter()
        .find(|definition| definition.id == adapter_id)
        .ok_or_else(|| format!("{adapter_id} is not registered"))?;

    if definition.generation == LocalGenerationKind::OllamaHttp {
        return generate_spec_with_ollama(prompt, references).await;
    }

    if definition.generation == LocalGenerationKind::AcpOnly {
        return Err(format!(
            "{} uses an ACP-style runtime. Strut detects it, but real generation is disabled until ACP transport support lands.",
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
    let prompt = local_character_prompt(prompt, references, reference_files.as_ref());
    let output = run_local_cli_command(
        &definition,
        &command,
        reference_dir,
        &prompt,
        Duration::from_secs(120),
    )?;
    let _ = reference_files
        .as_ref()
        .map(|files| fs::remove_dir_all(&files.directory));

    if !output.ok {
        return Err(command_output_preview(&output.stdout, &output.stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let assistant_text = cli_assistant_text(&stdout);
    parse_character_spec(&assistant_text)
        .or_else(|_| parse_character_spec(&stdout))
        .or_else(|_| parse_character_spec(&stderr))
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

fn cli_assistant_text(text: &str) -> String {
    let mut collected = String::new();
    for line in text.lines() {
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            collect_text_fields(&value, &mut collected);
        }
    }
    if collected.trim().is_empty() {
        text.to_string()
    } else {
        collected
    }
}

fn collect_text_fields(value: &Value, out: &mut String) {
    match value {
        Value::String(text) => {
            if text.contains("\"variant\"") || text.contains('{') {
                out.push_str(text);
                out.push('\n');
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_text_fields(value, out);
            }
        }
        Value::Object(map) => {
            for key in ["text", "content", "message", "delta", "output", "response"] {
                if let Some(value) = map.get(key) {
                    collect_text_fields(value, out);
                }
            }
        }
        _ => {}
    }
}

fn parse_character_spec(text: &str) -> Result<strut_core::CharacterSpec, String> {
    if let Ok(spec) = serde_json::from_str::<strut_core::CharacterSpec>(text.trim()) {
        return Ok(spec);
    }

    for json_text in extract_json_objects(text).into_iter().rev() {
        if let Ok(spec) = serde_json::from_str::<strut_core::CharacterSpec>(&json_text) {
            return Ok(spec);
        }
    }

    Err("model did not return a valid Strut character spec".to_string())
}

fn extract_json_objects(text: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, character) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(index);
                }
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = start.take() {
                        objects.push(text[start..=index].to_string());
                    }
                }
            }
            _ => {}
        }
    }

    objects
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            studio_status,
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
        assert!(ids.contains(&"opencode"));
        assert!(ids.contains(&"cursor-agent"));
        assert!(ids.contains(&"qwen"));
        assert!(ids.contains(&"qoder"));
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
    fn parses_json_spec_from_streaming_cli_output() {
        let text = cli_assistant_text(
            r##"{"type":"assistant","message":{"content":"{\"variant\":\"floating-helper\",\"name\":\"Stream Bot\",\"accent\":\"#51bfd0\",\"shell\":\"#f6f1e8\"}"}}"##,
        );
        let spec = parse_character_spec(&text).expect("spec should parse");

        assert_eq!(spec.variant, "floating-helper");
        assert_eq!(spec.name.as_deref(), Some("Stream Bot"));
    }

    #[test]
    fn provider_config_path_is_local() {
        let path = provider_config_path().expect("config path");
        assert!(path.ends_with("byok.json"));
    }

    #[test]
    fn endpoint_guard_allows_loopback_and_blocks_private_networks() {
        assert!(ensure_safe_endpoint("http://localhost:1234/v1").is_ok());
        assert!(ensure_safe_endpoint("http://127.0.0.1:11434").is_ok());
        assert!(ensure_safe_endpoint("https://api.openai.com/v1").is_ok());
        assert!(ensure_safe_endpoint("http://192.168.1.20:8080").is_err());
        assert!(ensure_safe_endpoint("ftp://api.example.com").is_err());
    }

    #[test]
    fn project_name_is_sanitized() {
        assert_eq!(
            sanitize_project_name("  My Bot / Demo!! ").expect("project name"),
            "My Bot Demo"
        );
    }
}
