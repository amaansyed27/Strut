# Use Local AI

Strut is designed to work without forcing your design files through a cloud service.

The first local provider target is Ollama.

## Why Use Local AI

- Keep private design files on your machine.
- Work without provider rate limits.
- Test smaller local models for simple edits.
- Use local GPU acceleration when available.

## Intended Flow

1. Install Ollama.
2. Pull a model that supports your workflow.
3. Open Strut Studio.
4. Select Ollama in the provider panel.
5. Run a Plan Mode or edit request.

Use the desktop app for this flow:

```powershell
npm run studio:dev
```

The browser preview cannot execute local CLI checks or call local model endpoints through Tauri.

## Local CLI Agents

Strut can detect local agent CLIs by checking the command on your `PATH` and running a real version command.

Current adapter checks include:

- Codex: `codex --version`
- Claude Code: `claude --version`
- Gemini CLI: `gemini --version`
- Copilot CLI: `gh --version`
- Antigravity: `antigravity --version`
- Kiro: `kiro --version`
- Ollama: `ollama --version`

Strut does not run arbitrary code-writing agents for generation until a command profile is configured. That prevents the app from launching an agent that might edit files without an explicit run profile.

## Privacy

Local AI does not automatically mean every workflow is private. Strut still needs to show which provider is being used and what context is included before a request runs.
