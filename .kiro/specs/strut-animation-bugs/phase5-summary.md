# Phase 5: Export Feature & Generation Testing

## Summary

Phase 5 addresses two user-reported issues:

1. **Missing Export Button** - Users cannot export animations for use in web/mobile apps
2. **Chat Generation Not Working** - When testing animation generation, the system returned conversational text instead of animations

## Issue Analysis

### Issue 1: Export Functionality Missing

**Status**: True bug - feature exists in CLI but not exposed in Studio UI

The `strut export react` command exists in the CLI and generates:
- `StrutAnimation.tsx` - React component with inline SVG
- `scene.json` - Complete Strut document JSON
- `README.md` - Usage instructions

**Solution**: Add Tauri command + UI button to expose this functionality

### Issue 2: Chat Generation "Not Working"

**Status**: Not a bug - Testing procedure issue

**Root Cause**: User tested with external Gemini chat, not through Studio UI

From the chat log provided:
```
Read lines 1-100 of 409 from src/prompts.rs
Read lines 1-100 of 801 from src/commands.rs
```

This shows Gemini operating as a general coding assistant, not using Strut's generation system.

**Why External Gemini Doesn't Work**:
1. External Gemini chat doesn't receive `GENERATION_PLAN_SYSTEM_PROMPT`
2. Without that prompt, Gemini doesn't know to return Strut JSON
3. It responds conversationally and tries to "help" by researching files

**Correct Testing Procedure**:
1. Open Strut Studio desktop app
2. Configure a provider (BYOK, Ollama, or local Gemini CLI)
3. Create/open a project
4. Use the STUDIO'S chat interface
5. Send: "Make me 3d rolling die"
6. Animation should appear in preview panel

**Classification Verification**:
The `classify_request_intent` function in `prompts.rs` already includes:
```rust
let generation_words = [
    "generate", "create", "make", "build", "animate", ...
];
```

So "Make me 3d rolling die" WILL classify correctly as `Generate`.

## Phase 5 Tasks

### Task 9: Export Feature (New Feature)
- Add Tauri command `export_animation_to_react`
- Add Export button in WorkspaceTopbar
- Create ExportDialog component
- Add export button to animation list items
- Show success notification with "Open folder" link

### Task 10: Generation Testing & UX Improvements (Documentation + Enhancement)
- Document correct testing procedure
- Add debug logging (STRUT_DEBUG_GENERATION env var)
- Verify classification works correctly (it should)
- Improve error messages when LLM returns wrong format
- Add UI improvements:
  - Placeholder text with examples
  - Quick example buttons ("Coin flip", "Dice roller", etc.)
  - Provider status indicator
  - Generation progress indicator

## Key Insights

1. **The generation code works correctly** - Phase 1-4 fixed the bugs
2. **Classification works correctly** - "make", "create", "build" trigger Generate mode
3. **Testing must use Studio UI** - External tools don't have Strut context
4. **Export is CLI-only** - Needs Studio UI integration

## Completed Implementation

1. Task 9 exposes React export through Studio and writes `StrutAnimation.tsx`, `scene.json`, and `README.md`.
2. Task 10 documents the correct Studio testing workflow, adds debug logging, improves provider JSON errors, and adds composer examples.
3. Generation intent now wins over chat-only context for imperative animation requests such as `Make me 3d rolling die`.

## Files to Modify

**Task 9 (Export)**:
- `apps/studio/src-tauri/src/commands.rs` - Add Tauri command
- `apps/studio/src-tauri/src/main.rs` - Register command
- `apps/studio/src/app/WorkspaceTopbar.tsx` - Add Export button
- `apps/studio/src/features/export/ExportDialog.tsx` - New component
- `apps/studio/src/App.tsx` - Add export to the preview Project animations list

**Task 10 (Debug + UX)**:
- `apps/studio/src-tauri/src/commands.rs` - Add debug logging
- `apps/studio/src/App.tsx` - Add placeholder text, examples UI
- `TESTING-GENERATION.md` - Add testing guide
- `README.md` - Add troubleshooting pointer

## Testing Checklist

After implementing Phase 5:

- [x] Export button visible in WorkspaceTopbar
- [x] Export dialog opens and shows format options
- [x] Export creates files in correct directory
- [x] Success state appears with "Open folder" action
- [x] Animation list items have export icon
- [x] Debug logging works when STRUT_DEBUG_GENERATION=1
- [x] Placeholder text shows generation examples
- [x] Quick example buttons fill input
- [x] Provider status indicator shows current provider
- [x] Generation through Studio UI routes explicit animation requests to generation
- [x] Error messages are clear when LLM returns wrong format
