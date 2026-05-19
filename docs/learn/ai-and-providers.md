# AI And Providers

Strut is AI-first, but it is not tied to one model company.

You can use cloud providers with your own keys, local models through tools like Ollama, or local coding agents that already run on your machine.

## Provider Types

### Cloud Models

Use these when you want strong reasoning, vision, or generation quality:

- OpenAI
- Anthropic
- Gemini
- OpenRouter
- Azure OpenAI
- OpenAI-compatible endpoints

### Local Models

Use these when you want local control:

- Ollama
- LM Studio
- local OpenAI-compatible servers

### Local Agents

Use these when you want Strut to coordinate with coding tools installed on your machine:

- Codex
- Claude Code
- Gemini CLI
- Antigravity
- Kiro
- Copilot CLI
- OpenCode
- Cursor-style agents

## How Strut Uses AI

Strut asks AI to create structured edits, not random images.

Reference images can be attached from the Studio chat composer. The desktop app keeps those images with the chat and passes them to the selected generation route:

- OpenAI-compatible providers receive image URL content blocks.
- Anthropic receives image content blocks.
- Gemini receives inline image parts.
- Ollama receives base64 image payloads through its local generate API.

Provider support still depends on the selected model. Use a vision-capable model when you expect Strut to read a raster mockup or character reference.

Good AI output:

```txt
create artboard
create layers
add keyframes
create state machine
bind runtime inputs
verify result
```

Bad AI output:

```txt
generate an opaque image that cannot be edited
```

The goal is always an editable animation component.

## Desktop Runtime

Provider features run in the Strut desktop app, not in the browser preview.

The Vite browser preview is useful for checking layout and basic interactions, but it cannot:

- run local CLI commands
- call BYOK providers from the Tauri backend
- save provider config on disk
- route generation through a local model or cloud provider

For real provider work, run:

```powershell
npm run studio:dev
```

## Current Provider Flow

In the desktop app, open **Providers** from the sidebar. The provider panel has two real generation modes.

**Local CLI** checks installed tools on your `PATH` and common tool directories. Strut invokes supported agents through stdin with a structured generation prompt, matching the pattern used by Open Design for CLIs such as Codex, Claude Code, Gemini CLI, OpenCode, Cursor Agent, Qwen, Qoder, Copilot CLI, and Ollama. ACP-only runtimes such as Kiro and Antigravity are detected, but generation stays disabled until Strut ships a real ACP transport.

**BYOK APIs** accepts a provider, API key, base URL, and model. Strut saves the provider endpoint/model locally and keeps the API key in the current app session. Test Connection makes a real structured generation request from the Tauri backend.

Character generation is routed through the selected provider only. Ollama generation uses the local Ollama API with image payloads. BYOK vision-capable providers receive attached reference images. Local CLIs receive the prompt plus temporary reference image file paths when references are attached.

There is no built-in fake generator. If no real provider is selected, credentials are missing, the CLI is not installed, or the browser preview is being used, Strut stops and shows the exact reason. The sidebar footer always shows the selected provider mode and the latest provider status before you generate.
