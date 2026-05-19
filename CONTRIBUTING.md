# Contributing to Strut

Strut is early. The most useful contributions are focused, well-tested changes that preserve the core product contract: open format, desktop-first editor, fast Rust core, AI-native workflows, and runtime-ready exports.

## Development Principles

- Keep the `.strut` format documented before relying on it in code.
- Keep AI output structured as document patches, not opaque generated blobs.
- Prefer deterministic import/export behavior before model-based guessing.
- Keep local-first behavior and user-owned keys as the default.
- Treat verifier failures as blockers for "done" claims.
- Make small commits that separate docs, scaffolding, core logic, UI, and tests.

## Commit Style

Use clear, scoped commit messages:

```txt
docs: define mvp checkpoints
core: add scene graph primitives
format: validate document manifest
studio: add timeline shell
agent: add ollama provider adapter
verifier: add state reachability check
```

## Pull Requests

Each pull request should include:

- What changed.
- What commands were run.
- Which manual review checkpoint applies.
- Any remaining risks or partial behavior.
- Screenshots or recordings for Studio UI changes.

## Documentation

Docs are product surface in Strut. If a change affects user workflows, file format, runtime APIs, provider behavior, local agent execution, permissions, or verification, update the relevant public document in `docs/learn`, `docs/guides`, or `docs/reference`. Maintainer-only process notes belong in `docs/internal`.
