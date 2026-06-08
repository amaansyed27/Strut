# Agentic Strut Integration

This guide shows the intended Phase 5 workflow for a coding agent integrating a Strut animation into another project.

## React

Create or locate a scene:

```powershell
cargo run -p strut-cli -- inspect scene .\animation\main.strut --json
```

Generate a sprite-python backed plan:

```powershell
cargo run -p strut-cli -- sprite plan "make a subtle success button microinteraction" --json --dry-run --explain > .\animation\plan.json
```

Validate before writing:

```powershell
cargo run -p strut-cli -- patch --scene .\animation\main.strut --from .\animation\plan.json --dry-run --json
```

Apply and verify:

```powershell
cargo run -p strut-cli -- patch --scene .\animation\main.strut --from .\animation\plan.json --json
cargo run -p strut-cli -- verify .\animation\main.strut --json
```

Export React files:

```powershell
cargo run -p strut-cli -- export react --scene .\animation\main.strut --out .\src\strut-animation --json
```

Use the component:

```tsx
import { StrutAnimation } from "./strut-animation/StrutAnimation";

export function SuccessButtonMotion() {
  return <StrutAnimation state="idle" />;
}
```

## Next.js

Export into a component folder:

```powershell
cargo run -p strut-cli -- export react --scene .\animation\main.strut --out .\app\components\strut-animation --json
```

Use the exported component from a client component:

```tsx
"use client";

import { StrutAnimation } from "./strut-animation/StrutAnimation";

export function HeroMotion() {
  return <StrutAnimation state="idle" />;
}
```

The Phase 5 export is static SVG markup plus `scene.json`; runtime playback is a future runtime integration layer.

## Plain Web Or Runtime-Web

For a plain web target, keep the validated scene as an asset:

```powershell
cargo run -p strut-cli -- verify .\public\motion\main.strut --json
cargo run -p strut-cli -- render --scene .\public\motion\main.strut --state idle --out .\public\motion\proof.svg --json --no-open
```

Use the render proof during review and ship the `.strut` package beside the app until `runtime-web` playback is wired to the full renderer.

## Agent Checklist

1. Inspect project files and scene contents.
2. Generate a plan with `--json --dry-run --explain`.
3. Save the plan JSON.
4. Patch with `--dry-run` first.
5. Apply only after validation passes.
6. Verify the resulting `.strut`.
7. Render a deterministic proof.
8. Export React files or hand the validated `.strut` to the target runtime.
9. Report exactly what changed and which limitations remain.

