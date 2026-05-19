# Runtime API

The runtime API is how apps control exported Strut animations.

This API is not implemented yet. It is the intended developer experience for the MVP.

## JavaScript Runtime

```ts
import { loadStrutUrl, mountMinimalBot } from "@strut/runtime-web";

const strutPackage = await loadStrutUrl("/samples/minimal-bot.strut");
const player = mountMinimalBot(stageElement, strutPackage.document, "idle");

player.setState("wave");
player.setState("celebrate");
```

## React Runtime

The React runtime is still planned. Its API should wrap the same runtime concepts:

```tsx
<Strut
  src="/login-button.strut"
  artboard="LoginButton"
  stateMachine="Interaction"
  inputs={{ status: "loading" }}
  bindings={{ label: "Sign in" }}
/>
```

## Runtime Rules

- Inputs must be named and stable.
- Missing inputs should fail clearly.
- Events should be typed.
- Runtime files should not require the desktop editor.
