# Testing Animation Generation in Strut

## Common Mistake

Do not test Strut generation by chatting with Gemini, Claude, or another external assistant. External chats do not receive Strut's router prompt, generation schema, project context, or repair prompts, so they usually answer conversationally instead of returning validated Strut animation JSON.

## Correct Testing Procedure

1. Open the Strut Studio desktop app.
2. Configure a provider from the Providers page: BYOK, Ollama, or a local CLI adapter such as Gemini CLI.
3. Create or open a project.
4. Start a Studio chat.
5. Send an animation request such as `Make me 3d rolling die`.
6. Verify that the animation appears in the preview panel and is added to Project animations.

Good smoke prompts:

```txt
Make me 3d rolling die
Create a bouncing ball animation
Build a coin flip with heads and tails
Generate a smooth loader animation
```

## Troubleshooting

### Provider Did Not Return Valid Strut Animation JSON

This means the provider returned prose, markdown, or malformed JSON after the initial generation and repair attempts.

Check:

- A provider is selected in Studio.
- The API key, endpoint, and model are correct for BYOK providers.
- Local adapters such as Gemini CLI or Ollama are installed and reachable.
- The prompt is an explicit generation request, for example `Create a bouncing ball`.
- A stricter model/provider can follow JSON-only instructions.

### Enable Debug Logging

Run Studio with:

```powershell
$env:STRUT_DEBUG_GENERATION='1'; npm run studio:dev
```

When enabled, the backend logs request classification, chat early-exit decisions, prompt previews, provider details, LLM response previews, parse results, and the final assistant result type.

## What Studio Sends

For `Make me 3d rolling die`, Studio should:

1. Classify the request as `Generate`.
2. Skip chat-only early exit because explicit generation intent wins.
3. Send `ASSISTANT_ROUTER_SYSTEM_PROMPT` plus `GENERATION_PLAN_SYSTEM_PROMPT`.
4. Parse or repair the provider response into a validated Strut document.
5. Render the result in preview and save it in Project animations.

External chat apps skip steps 2 through 5, which is why they are not a valid generation test.

## Quick Examples

The Studio composer includes quick prompt buttons:

- Coin flip: `Create a 3D coin flip animation`
- Dice roller: `Create a rolling dice with all 6 faces`
- Loader: `Create a smooth loader animation`
- Button: `Create a button with hover and press states`

## Exporting Animations

To use a generated animation in another app:

1. Click Export in the topbar, or click the export icon beside a saved Project animation.
2. Choose React Component.
3. Confirm the output directory. Relative paths are resolved inside the project folder and default to `exports/{animation-name}-react/`.
4. Click Export.
5. Use Open Export Folder from the success state.

Generated files:

- `StrutAnimation.tsx`
- `scene.json`
- `README.md`

Example:

```tsx
import { StrutAnimation } from "./StrutAnimation";

export function Example() {
  return <StrutAnimation state="idle" playAll />;
}
```

## Further Reading

- `apps/studio/src-tauri/src/prompts.rs`
- `apps/studio/src-tauri/src/commands.rs`
- `apps/studio/src-tauri/src/generation.rs`
- `.kiro/specs/strut-animation-bugs/`
