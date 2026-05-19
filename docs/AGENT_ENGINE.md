# Agent Engine

The Strut Agent Engine turns prompts, SVGs, images, sketches, and code context into validated Strut document edits.

## Principle

Agents do not directly write arbitrary final documents. Agents propose typed operations. Strut validates, previews, and verifies those operations before applying them.

```txt
Prompt or asset
  -> planning
  -> structured operations
  -> validation
  -> preview
  -> verifier
  -> accepted document patch
```

## Provider Support

BYOK providers:

- OpenAI
- Anthropic
- Gemini
- OpenRouter
- Azure OpenAI
- Ollama
- LM Studio
- OpenAI-compatible endpoints

Provider routing should support:

- text models
- vision models
- local models
- model capability detection
- per-task model preferences
- explicit context previews before cloud calls

## Local Agent Support

Local coding/design agents:

- Codex
- Claude Code
- Gemini CLI
- Antigravity
- Kiro
- Copilot CLI
- OpenCode
- Cursor-style agents
- custom shell adapters

Adapter interface:

```ts
interface StrutAgentAdapter {
  id: string;
  name: string;
  kind: "cloud-model" | "local-model" | "local-agent";
  detect(): Promise<DetectionResult>;
  capabilities(): AgentCapabilities;
  run(task: AgentTask, context: AgentContext): AsyncIterable<AgentEvent>;
  cancel(runId: string): Promise<void>;
}
```

## Strut Tools

Agents should use native Strut operations:

```txt
create_artboard
create_group
create_path
import_svg
create_timeline
add_keyframe
create_state_machine
add_input
add_transition
bind_property
emit_event
preview_animation
compare_snapshot
export_runtime
```

## Plan Mode

When the user has no mockup, Strut enters Plan Mode:

```txt
brief
  -> clarify constraints
  -> generate 2D sketches
  -> generate motion storyboard
  -> user review
  -> full Strut document
```

Plan Mode outputs should be lightweight and editable:

- rough vector sketches
- motion boards
- state diagrams
- timeline thumbnails
- interaction maps

## Mockup To Strut

SVG path:

```txt
SVG
  -> deterministic parser
  -> layer grouping
  -> semantic naming
  -> editable Strut scene graph
```

Raster path:

```txt
PNG/JPG/WebP
  -> image normalization
  -> segmentation
  -> OCR
  -> shape approximation
  -> vision-model semantic labeling
  -> editable Strut scene graph
```

## Security

- Keys stay local.
- Cloud model context must be visible before send.
- MCP starts read-only.
- Write operations require explicit tool permission.
- Local agent commands run with scoped workspace access.
- External files are treated as untrusted.
- Provider adapters must not silently forward private local context.
