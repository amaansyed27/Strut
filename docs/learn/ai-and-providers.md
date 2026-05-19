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
