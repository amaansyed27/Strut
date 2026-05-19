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

The desktop app should open with:

- a layer list
- a central stage
- a sample animated bot
- a timeline
- state-machine controls
- an agent panel

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

Click the state buttons in the left panel:

- Idle
- Hover
- Pressed
- Loading
- Success
- Error

This first shell is not the final editor. It is the visible frame for the core workflows that are being built next.
