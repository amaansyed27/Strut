# GitHub Topics

GitHub topics cannot be committed directly into a repository unless they are set through the GitHub UI, GitHub CLI, or API after the remote repository exists. This file is the canonical topic list for Strut.

## Primary Topics

```txt
animation
motion-design
vector-graphics
rive-alternative
lottie
tauri
rust
wgpu
webgpu
ai-design
generative-design
local-first
byok
mcp
ollama
openai
anthropic
gemini
openrouter
react
desktop-app
```

## Optional Topics

```txt
state-machines
interactive-animation
design-tools
creative-coding
svg
wasm
typescript
cross-platform
agentic-ai
local-ai
gpu-acceleration
desktop-software
open-source
```

## GitHub CLI Command

After the repository exists and `gh` is installed/authenticated:

```powershell
gh repo edit OWNER/strut --add-topic animation --add-topic motion-design --add-topic vector-graphics --add-topic rive-alternative --add-topic lottie --add-topic tauri --add-topic rust --add-topic wgpu --add-topic webgpu --add-topic ai-design --add-topic generative-design --add-topic local-first --add-topic byok --add-topic mcp --add-topic ollama --add-topic openai --add-topic anthropic --add-topic gemini --add-topic openrouter --add-topic react --add-topic desktop-app
```

## Why These Topics

The topics target three search paths:

- Designers looking for motion and vector tooling.
- Engineers looking for Rust/Tauri/WebGPU desktop software.
- AI builders looking for BYOK, local agent, and Ollama-compatible workflows.
