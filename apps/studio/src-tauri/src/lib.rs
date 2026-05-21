use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CHARACTER_DOCUMENT_SYSTEM_PROMPT: &str = r##"You convert design prompts into editable Strut motion documents. Return only JSON in this shape: {"document": <StrutDocument>}.

StrutDocument schema:
- id: UUID string
- name: character or scene name
- artboards: one artboard with id, name, width 960, height 540, and editable nodes
- nodes: recursive objects with id, name, kind, transform, style, shape, children
- kind: group, rect, ellipse, path, text, image, or hit_area
- transform: translate_x, translate_y, rotate, scale_x, scale_y
- style: fill, stroke, stroke_width, opacity, linecap, linejoin
- shape variants use {"type":"rect","x":...,"y":...,"width":...,"height":...,"rx":...}, {"type":"ellipse","cx":...,"cy":...,"rx":...,"ry":...}, {"type":"path","d":"SVG path data"}, {"type":"text","x":...,"y":...,"value":"...","size":...}, or {"type":"none"}
- timelines: include idle_float, wave, blink, scan, and celebrate timelines. Every track target must be a real node id.
- state_machines: include one machine with states idle, float, wave, blink, scan, celebrate, sleep.
- bindings and events: arrays, may be empty.

Make the silhouette, named layers, palette, and motion match the request. If the user asks for Pikachu, R2D2, a bird, a loader, or anything else, create that subject's distinct editable parts. Do not return a preset selector such as variant/name/accent/shell."##;

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
            let document = generate_document_with_byok(&prompt, &config, &references).await?;
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
            let document =
                generate_document_with_local_adapter(&adapter_id, &prompt, &references).await?;
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
        let prompt = local_character_prompt(
            "Create a small floating helper named Smoke Bot with a cyan accent.",
            &[],
            None,
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
    let smoke_prompt =
        "Create a small floating helper character named Smoke Bot with a cyan accent.";
    match generate_document_with_byok(smoke_prompt, &config, &[]).await {
        Ok(_) => Ok(ProviderOperationResult {
            ok: true,
            status: format!("{} ready", provider_label(&config.provider_id)),
            detail: "provider completed a real Strut document generation smoke test".to_string(),
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
        if Path::new(command).extension().is_some() {
            return vec![PathBuf::from(command)];
        }
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

        let mut candidates = extensions
            .into_iter()
            .flat_map(|extension| {
                [
                    PathBuf::from(format!("{command}{extension}")),
                    PathBuf::from(format!("{command}{}", extension.to_lowercase())),
                ]
            })
            .collect::<Vec<_>>();
        candidates.push(PathBuf::from(command));
        candidates
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
        &local_character_prompt(
            "Create a small floating helper named Smoke Bot with a cyan accent.",
            &[],
            None,
        ),
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
            "--prompt".to_string(),
            "Generate exactly the requested JSON from stdin.".to_string(),
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
        "{CHARACTER_DOCUMENT_SYSTEM_PROMPT}\n\nUser request:\n{}",
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
    text.push_str(
        "\n\nDo not inspect files, run tools, edit the workspace, or explain your answer. Return only the JSON object. Do not include markdown or a legacy variant spec.",
    );
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

async fn generate_document_with_byok(
    prompt: &str,
    config: &ByokProviderConfig,
    references: &[ReferenceImageInput],
) -> Result<strut_core::Document, String> {
    ensure_byok_config(config)?;
    let response_text = match config.provider_id.as_str() {
        "anthropic" => anthropic_message(prompt, config, references).await?,
        "gemini" => gemini_generate_content(prompt, config, references).await?,
        _ => openai_compatible_chat(prompt, config, references).await?,
    };

    parse_generated_document(&response_text)
}

async fn generate_document_with_local_adapter(
    adapter_id: &str,
    prompt: &str,
    references: &[ReferenceImageInput],
) -> Result<strut_core::Document, String> {
    let definition = local_adapter_definitions()
        .into_iter()
        .find(|definition| definition.id == adapter_id)
        .ok_or_else(|| format!("{adapter_id} is not registered"))?;

    if definition.generation == LocalGenerationKind::OllamaHttp {
        return generate_document_with_ollama(prompt, references).await;
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
        Duration::from_secs(240),
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
    parse_generated_document(&assistant_text)
        .or_else(|_| parse_generated_document(&stdout))
        .or_else(|_| parse_generated_document(&stderr))
}

async fn generate_document_with_ollama(
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
            "prompt": format!("{CHARACTER_DOCUMENT_SYSTEM_PROMPT}\nPrompt: {}", prompt_with_reference_context(prompt, references)),
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
    parse_generated_document(text)
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
                {"role": "system", "content": CHARACTER_DOCUMENT_SYSTEM_PROMPT},
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
            "max_tokens": 4096,
            "system": CHARACTER_DOCUMENT_SYSTEM_PROMPT,
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
        "text": format!("{CHARACTER_DOCUMENT_SYSTEM_PROMPT}\nPrompt: {}", prompt_with_reference_context(prompt, references))
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
            out.push_str(text);
        }
        Value::Array(values) => {
            for value in values {
                collect_text_fields(value, out);
            }
        }
        Value::Object(map) => {
            if map.get("role").and_then(Value::as_str) == Some("user") {
                return;
            }
            for key in ["text", "content", "message", "delta", "output", "response"] {
                if let Some(value) = map.get(key) {
                    collect_text_fields(value, out);
                }
            }
        }
        _ => {}
    }
}

fn parse_generated_document(text: &str) -> Result<strut_core::Document, String> {
    if let Ok(document) = serde_json::from_str::<strut_core::Document>(text.trim()) {
        return validate_generated_document(document);
    }

    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        if let Ok(document) = document_from_value(&value) {
            return validate_generated_document(document);
        }
    }

    let mut last_error = None;
    for json_text in extract_json_objects(text).into_iter().rev() {
        match serde_json::from_str::<Value>(&json_text)
            .map_err(|error| error.to_string())
            .and_then(|value| document_from_value(&value))
            .and_then(validate_generated_document)
        {
            Ok(document) => return Ok(document),
            Err(error) => last_error = Some(error),
        }
    }

    if parse_character_spec(text).is_ok()
        || extract_json_objects(text)
            .iter()
            .any(|json_text| parse_character_spec(json_text).is_ok())
    {
        return Err(
            "provider returned the old preset spec format. Strut now requires a full editable document so different prompts do not collapse into the same character.".to_string(),
        );
    }

    Err(last_error.unwrap_or_else(|| "model did not return a valid Strut document".to_string()))
}

fn document_from_value(value: &Value) -> Result<strut_core::Document, String> {
    let mut document_value = value
        .get("document")
        .cloned()
        .unwrap_or_else(|| value.clone());
    normalize_generated_document_value(&mut document_value);
    serde_json::from_value::<strut_core::Document>(document_value)
        .map_err(|error| format!("model response was not a Strut document: {error}"))
}

fn normalize_generated_document_value(value: &mut Value) {
    let mut id_map = HashMap::new();
    let mut next_id = 1u128;
    normalize_generated_ids(value, &mut id_map, &mut next_id);
    fill_generated_defaults(value);
}

fn normalize_generated_ids(
    value: &mut Value,
    id_map: &mut HashMap<String, String>,
    next_id: &mut u128,
) {
    match value {
        Value::Array(values) => {
            for value in values {
                normalize_generated_ids(value, id_map, next_id);
            }
        }
        Value::Object(map) => {
            for key in ["id", "target"] {
                if key == "id"
                    && (map.contains_key("timeline") || map.contains_key("timeline_id"))
                    && !map.contains_key("kind")
                {
                    continue;
                }
                if let Some(Value::String(raw_id)) = map.get_mut(key) {
                    if !looks_like_uuid(raw_id) {
                        let strict_id = id_map.entry(raw_id.clone()).or_insert_with(|| {
                            let id = format!("00000000-0000-0000-0000-{next_id:012x}");
                            *next_id += 1;
                            id
                        });
                        *raw_id = strict_id.clone();
                    }
                }
            }
            for value in map.values_mut() {
                normalize_generated_ids(value, id_map, next_id);
            }
        }
        _ => {}
    }
}

fn fill_generated_defaults(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                fill_generated_defaults(value);
            }
        }
        Value::Object(map) => {
            if map.contains_key("artboards") {
                map.entry("timelines").or_insert_with(|| json!([]));
                map.entry("state_machines").or_insert_with(|| json!([]));
                map.entry("bindings").or_insert_with(|| json!([]));
                map.entry("events").or_insert_with(|| json!([]));
            }

            if map.contains_key("tracks") {
                if let Some(duration) = map.remove("duration") {
                    map.entry("duration_ms").or_insert(duration);
                }
            }

            if map.contains_key("keyframes") {
                if let Some(property) = map.get_mut("property") {
                    normalize_track_property(property);
                }
            }

            if (map.contains_key("time") || map.contains_key("time_ms"))
                && map.contains_key("value")
            {
                if let Some(time) = map.remove("time") {
                    map.entry("time_ms").or_insert(time);
                }
                if let Some(value) = map.get_mut("value") {
                    normalize_keyframe_value(value);
                }
                if let Some(easing) = map.get_mut("easing") {
                    normalize_easing(easing);
                } else {
                    map.insert("easing".to_string(), json!("ease_in_out"));
                }
            }

            if map.contains_key("states") {
                map.entry("name").or_insert_with(|| json!("GeneratedMoods"));
                map.entry("inputs").or_insert_with(|| json!([]));
                map.entry("transitions").or_insert_with(|| json!([]));
                if let Some(states) = map.get_mut("states") {
                    normalize_state_list(states);
                }
            }

            if map.contains_key("kind") {
                map.entry("transform")
                    .or_insert_with(|| json!(default_transform_value()));
                map.entry("style")
                    .or_insert_with(|| json!(default_style_value()));
                map.entry("shape")
                    .or_insert_with(|| json!({"type": "none"}));
                map.entry("children").or_insert_with(|| json!([]));
            }

            if let Some(style) = map.get_mut("style") {
                fill_style_defaults(style);
            }
            if let Some(transform) = map.get_mut("transform") {
                fill_transform_defaults(transform);
            }

            for value in map.values_mut() {
                fill_generated_defaults(value);
            }
        }
        _ => {}
    }
}

fn normalize_track_property(value: &mut Value) {
    let Value::String(property) = value else {
        return;
    };
    *property = match property.as_str() {
        "translate_x" | "translation_x" | "x" => "translation.x",
        "translate_y" | "translation_y" | "y" => "translation.y",
        "rotate" => "rotation",
        "scale_x" => "scale.x",
        "scale_y" => "scale.y",
        other => other,
    }
    .to_string();
}

fn normalize_keyframe_value(value: &mut Value) {
    match value {
        Value::Number(_) => {
            let number = value.clone();
            *value = json!({"type": "number", "value": number});
        }
        Value::String(text) if text.starts_with('#') => {
            let color = text.clone();
            *value = json!({"type": "color", "value": color});
        }
        Value::String(_) => {
            let text = value.clone();
            *value = json!({"type": "text", "value": text});
        }
        Value::Object(map) if !map.contains_key("type") => {
            if map.contains_key("x") && map.contains_key("y") {
                let point = value.clone();
                *value = json!({"type": "point", "value": point});
            }
        }
        _ => {}
    }
}

fn normalize_easing(value: &mut Value) {
    let Value::String(easing) = value else {
        *value = json!("ease_in_out");
        return;
    };
    *easing = match easing.as_str() {
        "easeIn" | "easeInQuad" | "ease_in_quad" | "ease-in" => "ease_in",
        "easeOut" | "easeOutQuad" | "ease_out_quad" | "ease-out" => "ease_out",
        "easeInOut" | "easeInOutQuad" | "ease_in_out_quad" | "ease-in-out" => "ease_in_out",
        "linear" => "linear",
        "ease_in" | "ease_out" | "ease_in_out" => easing.as_str(),
        _ => "ease_in_out",
    }
    .to_string();
}

fn normalize_state_list(value: &mut Value) {
    let Value::Array(states) = value else {
        return;
    };
    for state in states {
        if let Value::Object(map) = state {
            let name = map
                .get("name")
                .or_else(|| map.get("id"))
                .and_then(Value::as_str)
                .unwrap_or("idle")
                .to_lowercase()
                .replace(' ', "_");
            *state = Value::String(name);
        }
    }
}

fn fill_style_defaults(value: &mut Value) {
    let Value::Object(map) = value else {
        *value = json!(default_style_value());
        return;
    };
    normalize_none_string(map.get_mut("fill"));
    normalize_none_string(map.get_mut("stroke"));
    map.entry("fill").or_insert(Value::Null);
    map.entry("stroke").or_insert(Value::Null);
    map.entry("stroke_width").or_insert(json!(0.0));
    map.entry("opacity").or_insert(json!(1.0));
    map.entry("linecap").or_insert(json!("round"));
    map.entry("linejoin").or_insert(json!("round"));
}

fn fill_transform_defaults(value: &mut Value) {
    let Value::Object(map) = value else {
        *value = json!(default_transform_value());
        return;
    };
    map.entry("translate_x").or_insert(json!(0.0));
    map.entry("translate_y").or_insert(json!(0.0));
    map.entry("rotate").or_insert(json!(0.0));
    map.entry("scale_x").or_insert(json!(1.0));
    map.entry("scale_y").or_insert(json!(1.0));
}

fn normalize_none_string(value: Option<&mut Value>) {
    if let Some(value) = value {
        if let Value::String(text) = value {
            if text.eq_ignore_ascii_case("none") || text.eq_ignore_ascii_case("transparent") {
                *value = Value::Null;
            }
        }
    }
}

fn default_style_value() -> Value {
    json!({
        "fill": null,
        "stroke": null,
        "stroke_width": 0.0,
        "opacity": 1.0,
        "linecap": "round",
        "linejoin": "round"
    })
}

fn default_transform_value() -> Value {
    json!({
        "translate_x": 0.0,
        "translate_y": 0.0,
        "rotate": 0.0,
        "scale_x": 1.0,
        "scale_y": 1.0
    })
}

fn looks_like_uuid(value: &str) -> bool {
    value.len() == 36
        && value
            .chars()
            .enumerate()
            .all(|(index, character)| match index {
                8 | 13 | 18 | 23 => character == '-',
                _ => character.is_ascii_hexdigit(),
            })
}

fn validate_generated_document(
    document: strut_core::Document,
) -> Result<strut_core::Document, String> {
    strut_format::validate_document(&document)
        .map_err(|error| format!("generated Strut document failed validation: {error}"))?;
    if document.timelines.is_empty() {
        return Err("generated Strut document must include timelines".to_string());
    }
    if document.state_machines.is_empty() {
        return Err("generated Strut document must include a state machine".to_string());
    }
    if document
        .state_machines
        .iter()
        .all(|machine| !machine.states.iter().any(|state| state == "idle"))
    {
        return Err("generated Strut document must include an idle state".to_string());
    }
    if count_document_nodes(&document) < 6 {
        return Err(
            "generated Strut document must contain at least six editable nodes".to_string(),
        );
    }
    Ok(document)
}

fn count_document_nodes(document: &strut_core::Document) -> usize {
    document
        .artboards
        .iter()
        .map(|artboard| count_nodes(&artboard.nodes))
        .sum()
}

fn count_nodes(nodes: &[strut_core::Node]) -> usize {
    nodes
        .iter()
        .map(|node| 1 + count_nodes(&node.children))
        .sum()
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

    fn collect_layer_names<'a>(nodes: &'a [strut_core::Node], names: &mut Vec<&'a str>) {
        for node in nodes {
            names.push(node.name.as_str());
            collect_layer_names(&node.children, names);
        }
    }

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
    fn parses_full_document_from_model_text() {
        let document_json = serde_json::to_string(&strut_core::Document::sample_owl_mascot())
            .expect("document json");
        let document =
            parse_generated_document(&format!("Here is JSON: {{\"document\":{document_json}}}"))
                .expect("document should parse");

        assert_eq!(document.name, "Owl Mascot");
        assert_eq!(document.artboards[0].name, "OwlMascot");
    }

    #[test]
    fn rejects_legacy_preset_spec_from_model_text() {
        let error = parse_generated_document(
            r##"{"variant":"owl-guide","name":"Owl Mascot","accent":"#78d64b","shell":"#8ee15a"}"##,
        )
        .expect_err("legacy spec should be rejected");

        assert!(error.contains("old preset spec format"));
    }

    #[test]
    fn normalizes_model_friendly_ids_and_partial_styles() {
        let document = parse_generated_document(
            r##"{
              "document": {
                "id": "doc",
                "name": "Loose Mascot",
                "artboards": [{
                  "id": "main-board",
                  "name": "Loose",
                  "width": 960,
                  "height": 540,
                  "nodes": [{
                    "id": "rig",
                    "name": "Rig",
                    "kind": "group",
                    "children": [
                      {"id":"body","name":"Body","kind":"ellipse","style":{"fill":"#78c137"},"shape":{"type":"ellipse","cx":480,"cy":270,"rx":100,"ry":120}},
                      {"id":"face","name":"Face","kind":"rect","style":{"fill":"none","stroke":"#111"},"shape":{"type":"rect","x":400,"y":220,"width":160,"height":90,"rx":24}},
                      {"id":"eye-a","name":"EyeA","kind":"ellipse","shape":{"type":"ellipse","cx":440,"cy":250,"rx":10,"ry":10}},
                      {"id":"eye-b","name":"EyeB","kind":"ellipse","shape":{"type":"ellipse","cx":520,"cy":250,"rx":10,"ry":10}},
                      {"id":"smile","name":"Smile","kind":"path","shape":{"type":"path","d":"M450 290 C480 315 510 290"}}
                    ]
                  }]
                }],
                "timelines": [{"id":"wave-line","name":"wave","duration":800,"tracks":[{"target":"rig","property":"translate_y","keyframes":[{"time":0,"value":0,"easing":"easeOutQuad"},{"time":400,"value":8,"easing":"easeInOutQuad"}]}]}],
                "state_machines": [{"id":"moods","name":"Moods","states":[{"id":"idle","name":"Idle"},{"id":"wave","name":"Wave"}],"transitions":[]}],
                "bindings": [],
                "events": []
              }
            }"##,
        )
        .expect("loose document should normalize");

        assert_eq!(document.name, "Loose Mascot");
        assert_eq!(
            document.timelines[0].tracks[0].target,
            document.artboards[0].nodes[0].id
        );
        assert_eq!(document.timelines[0].duration_ms, 800);
        assert_eq!(document.timelines[0].tracks[0].property, "translation.y");
        assert_eq!(document.timelines[0].tracks[0].keyframes[0].time_ms, 0);
        assert_eq!(
            document.artboards[0].nodes[0].children[0].style.opacity,
            1.0
        );
        assert_eq!(document.artboards[0].nodes[0].children[1].style.fill, None);
        assert!(document.state_machines[0]
            .states
            .contains(&"wave".to_string()));
    }

    #[test]
    fn parses_full_document_from_streaming_cli_output() {
        let document_text =
            serde_json::json!({"document": strut_core::Document::sample_minimal_bot()}).to_string();
        let text = cli_assistant_text(
            &serde_json::json!({"type":"assistant","message":{"content":document_text}})
                .to_string(),
        );
        let document = parse_generated_document(&text).expect("document should parse");

        assert_eq!(document.name, "Minimal Bot");
    }

    #[test]
    fn parses_full_document_from_gemini_delta_chunks() {
        let document_text =
            serde_json::json!({"document": strut_core::Document::sample_owl_mascot()}).to_string();
        let split_at = document_text.len() / 2;
        let (first, second) = document_text.split_at(split_at);
        let text = cli_assistant_text(&format!(
            "{}\n{}\n{}",
            serde_json::json!({"type":"message","role":"user","content":"Return a full Strut document."}),
            serde_json::json!({"type":"message","role":"assistant","content":first,"delta":true}),
            serde_json::json!({"type":"message","role":"assistant","content":second,"delta":true})
        ));
        let document = parse_generated_document(&text).expect("document should parse");

        assert_eq!(document.state_machines[0].name, "OwlMoods");
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
    fn windows_command_candidates_prefer_executable_shims() {
        let candidates = command_candidates("gemini");
        if cfg!(windows) {
            let first = candidates
                .first()
                .and_then(|path| path.extension())
                .and_then(|extension| extension.to_str())
                .unwrap_or_default()
                .to_lowercase();
            assert_ne!(first, "");
        } else {
            assert_eq!(candidates, vec![PathBuf::from("gemini")]);
        }
    }

    #[test]
    #[ignore = "requires authenticated Gemini CLI"]
    fn gemini_cli_generates_owl_mascot_end_to_end() {
        let document = tauri::async_runtime::block_on(generate_document_with_local_adapter(
            "gemini-cli",
            "Make a friendly owl style mascot like Duo. Keep it simple and editable, with wave, blink, scan, and celebrate animation states.",
            &[],
        ))
        .expect("Gemini CLI should return a full Strut document");
        let mut layer_names = Vec::new();
        collect_layer_names(&document.artboards[0].nodes, &mut layer_names);
        let states = &document.state_machines[0].states;

        assert!(document.name.to_lowercase().contains("owl"));
        assert!(document.artboards[0].nodes.len() >= 3 || layer_names.len() >= 6);
        assert!(layer_names
            .iter()
            .any(|name| name.to_lowercase().contains("owl")));
        for state in ["wave", "blink", "scan", "celebrate"] {
            assert!(states.iter().any(|item| item == state));
        }
    }

    #[test]
    fn project_name_is_sanitized() {
        assert_eq!(
            sanitize_project_name("  My Bot / Demo!! ").expect("project name"),
            "My Bot Demo"
        );
    }
}
