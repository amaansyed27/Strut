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

The desktop app opens on an empty Strut home screen. The left sidebar is where you move between your projects and chats. Nothing is seeded for you: create a project, pick a folder, then start a chat inside that project.

The main area starts in **Chat only** mode. Use the view switcher at the top to move between **Chat only**, **Chat + preview**, and **Editor**.

To create a project:

1. Click **New project** in the sidebar.
2. Name the project.
3. Choose or enter a location.
4. Click **Create project**.

Strut creates a project folder with:

- `strut.project.json`
- `scenes/starter.strut.json`
- `assets/`
- `exports/`

After that, Strut adds the project to the sidebar and opens a project chat. Use **New chat** for a fresh conversation, or the plus button on a project to start a chat inside that project.

Chats stay in the sidebar and are restored when you reopen the Studio preview. Use the delete button beside a chat to remove only that chat, or the remove button beside a project to remove the project from the sidebar.

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

In the chat box, ask Strut to make a character. You can also attach reference images with the **Reference** button in the composer. PNG, JPEG, WebP, GIF, and SVG references are shown in the conversation and passed to the desktop generation pipeline.

```txt
make a minimalist waving robot character like the reference image
make an owl mascot like Duo from Duolingo
```

Then open:

- **Chat + preview** to see the animated character beside the conversation.
- **Editor** to inspect project files, layers, parts, state machines, and motion states.
- **Providers** in the sidebar to select local CLI tools, Ollama, or BYOK providers.
- **Settings** in the sidebar footer to change appearance, workspace, generation, and editor defaults. Appearance supports **Auto**, **Light**, and **Dark**.

The browser preview is only for frontend inspection. Use the desktop app for real project creation, local CLI checks, BYOK provider calls, and provider-routed generation.
