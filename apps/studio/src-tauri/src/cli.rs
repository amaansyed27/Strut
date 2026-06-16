use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::process::{Command, Stdio};
use std::io::Write;
use crate::*;

pub fn resolve_adapter_command(definition: &LocalAdapterDefinition) -> Option<PathBuf> {
    definition
        .commands
        .iter()
        .find_map(|command| resolve_command(command))
}

pub fn resolve_command(command: &str) -> Option<PathBuf> {
    let candidates = command_candidates(command);
    command_search_dirs().into_iter().find_map(|path| {
        candidates
            .iter()
            .map(|candidate| path.join(candidate))
            .find(|candidate| candidate.is_file())
    })
}

pub fn command_search_dirs() -> Vec<PathBuf> {
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

pub fn command_candidates(command: &str) -> Vec<PathBuf> {
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

pub fn command_output_preview(stdout: &[u8], stderr: &[u8]) -> String {
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
pub fn open_folder_in_file_manager(folder: &Path) -> Result<(), String> {
    let status = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(folder).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(folder).spawn()
    } else {
        Command::new("xdg-open").arg(folder).spawn()
    }
    .map_err(|error| format!("Could not open project folder: {error}"))?
    .wait()
    .map_err(|error| format!("Could not open project folder: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("File explorer exited with status {status}"))
    }
}

pub fn run_ollama_smoke_test(command: &Path) -> Result<CommandRun, String> {
    run_command_with_stdin(
        command,
        &["run".to_string(), "llama3.2".to_string()],
        &[],
        None,
        &contextual_generation_prompt(
            "Create a small floating helper named Smoke Bot with a cyan accent.",
            None,
            GenerationStrategy::ProviderPlan,
        ),
        Duration::from_secs(60),
    )
}

pub fn run_local_cli_command(
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

pub fn run_local_cli_chat_command(
    definition: &LocalAdapterDefinition,
    command: &Path,
    input: &str,
    timeout: Duration,
) -> Result<CommandRun, String> {
    let args = local_chat_args(definition);
    let env = local_generation_env(definition);
    run_command_with_stdin(command, &args, &env, None, input, timeout)
}

pub fn run_command_with_stdin(
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

pub fn local_generation_args(
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

pub fn local_chat_args(definition: &LocalAdapterDefinition) -> Vec<String> {
    match definition.id {
        "codex" => vec![
            "exec".to_string(),
            "--json".to_string(),
            "--skip-git-repo-check".to_string(),
        ],
        "gemini-cli" => vec![
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--prompt".to_string(),
            "Answer the Strut user's message from stdin in concise markdown. Do not emit JSON unless asked.".to_string(),
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

pub fn local_generation_env(definition: &LocalAdapterDefinition) -> Vec<(&'static str, &'static str)> {
    match definition.id {
        "gemini-cli" => vec![("GEMINI_CLI_TRUST_WORKSPACE", "true")],
        _ => Vec::new(),
    }
}
