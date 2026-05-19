# Quick Start

This page gets you from a fresh checkout to the first Strut Studio preview.

## Requirements

Install:

- Rust
- Node.js
- npm

Strut currently uses Tauri, so the desktop app also needs the normal Tauri system dependencies for your operating system.

## Run Strut Studio

From the repository root:

```powershell
npm install
npm run check
npm run test
npm run studio:dev
```

The desktop app opens on the Strut home screen.

Create a project first:

1. Name the project.
2. Choose or enter a location.
3. Click **Create project**.

Strut creates a project folder with:

- `strut.project.json`
- `scenes/starter.strut.json`
- `assets/`
- `exports/`

After that, the main workspace opens with chat in the center, files on the left, and preview/editor controls on the right.

## Preview In The Browser During Development

When you only want to inspect the frontend shell:

```powershell
npm --workspace @strut/studio run dev -- --host 127.0.0.1 --port 1420
```

Then open:

```txt
http://127.0.0.1:1420
```

## What To Try First

In the chat box, ask Strut to make a character:

```txt
make a minimalist waving robot character like the reference image
make an owl mascot like Duo from Duolingo
```

Then open:

- **Files** to see project files.
- **Editor** to inspect layers and timelines.
- **AI** to select the built-in planner, local CLI tools, or BYOK providers.

The browser preview is only for frontend inspection. Use the desktop app for real project creation, local CLI checks, BYOK provider calls, and provider-routed generation.
