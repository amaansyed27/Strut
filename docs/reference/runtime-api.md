# Runtime API

The runtime API is how apps control exported Strut animations.

This API is not implemented yet. It is the intended developer experience for the MVP.

## JavaScript Runtime

```ts
import { loadStrutUrl, mountStrut } from "@strut/runtime-web";

const strutPackage = await loadStrutUrl("/samples/minimal-bot.strut");
const player = mountStrut(stageElement, strutPackage.document, {
  artboard: "MinimalBot",
  stateMachine: "BotMoods",
  initialState: "idle",
});

player.setState("wave");
player.setState("celebrate");
player.setInput("scan", true);
```

## React Runtime

The React runtime wraps the same runtime concepts:

```tsx
import { Strut } from "@strut/react";

export function Mascot() {
  return (
    <Strut
      src="/mascot.strut"
      artboard="Mascot"
      stateMachine="Interaction"
      state="wave"
      inputs={{ mood: "happy" }}
      bindings={{ label: "Welcome" }}
    />
  );
}
```

For Next.js client components, import from `@strut/next`:

```tsx
import { Strut } from "@strut/next";
```

## Runtime Rules

- Inputs must be named and stable.
- Missing inputs should fail clearly.
- Events should be typed.
- Runtime files should not require the desktop editor.
- Runtime rendering must use document nodes, shapes, styles, and timelines rather than a hardcoded preview template.
