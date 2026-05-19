# Manual Review Guide

Strut is built in reviewable milestones. Each milestone has commands to run and files to inspect before approving the next slice of work.

## Checkpoint 1: Docs And Repository Identity

Run:

```powershell
git status --short
git log --oneline --decorate -5
```

Inspect:

```txt
README.md
docs/ARCHITECTURE.md
docs/MVP_SCOPE.md
docs/STRUT_FORMAT.md
docs/AGENT_ENGINE.md
docs/GITHUB_TOPICS.md
assets/brand/strut-logo.svg
```

Check:

- The README clearly explains what Strut is.
- The MVP scope is narrow enough to build.
- Manual review checkpoints are visible.
- The file format has a concrete starting shape.
- The agent engine supports BYOK providers, Ollama, and local coding agents.

## Checkpoint 2: Rust/Tauri Scaffold

Run:

```powershell
npm install
npm run check
cargo test --workspace
npm run studio:dev
```

Inspect:

```txt
apps/studio/
Cargo.toml
package.json
crates/
```

Check:

- The Studio opens on desktop.
- Root scripts work from the repository root.
- The Rust workspace is split into crates instead of one large app.
- Generated caches and build outputs are ignored by git.

## Checkpoint 3: Open Strut Format

Run:

```powershell
npm run check
cargo test --workspace
cargo run -p strut-format --example validate samples/login-button.strut
```

Inspect:

```txt
crates/strut-format/
crates/strut-core/
samples/
docs/STRUT_FORMAT.md
```

Check:

- The sample file validates.
- Unsupported schema versions fail clearly.
- Document ids and names are stable.
- The schema matches the docs.

## Checkpoint 4: Runtime Preview

Run:

```powershell
npm run studio:dev
npm run runtime:example
```

Check:

- The sample animation renders.
- Timeline playback works.
- State machine inputs can be changed.
- Events are visible in the preview panel.

## Checkpoint 5: AI Provider And Local Agent Adapters

Run:

```powershell
npm run check
cargo test --workspace
cargo run -p strut-agent --example detect
```

Check:

- Ollama detection works when Ollama is installed.
- OpenAI-compatible provider configuration validates without sending a request.
- Local agent adapters report detected/not detected status.
- No API key is committed to the repository.

## Checkpoint 6: Mockup/SVG To Strut

Run:

```powershell
cargo run -p strut-import --example import_svg samples/assets/login-button.svg
npm run studio:dev
```

Check:

- SVG import creates editable layers.
- Layer names are usable.
- Imported shapes render close to the source.
- The verifier reports differences instead of silently accepting poor output.

## Checkpoint 7: Plan Mode

Run:

```powershell
npm run studio:dev
```

Check:

- A prompt without a mockup creates 2D sketch options first.
- The user can pick a direction before full generation.
- The final output remains editable.
- The verifier checks generated animations before export.
