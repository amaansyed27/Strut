# Export To React

Strut animations are designed to be controlled by app code.

The React runtime is not implemented yet, but this is the intended shape:

```tsx
import { Strut } from "@strut/react";

export function LoginButton() {
  return (
    <Strut
      src="/login-button.strut"
      artboard="LoginButton"
      stateMachine="Interaction"
      inputs={{
        status: "loading",
      }}
      onEvent={(event) => {
        console.log(event.name);
      }}
    />
  );
}
```

## Runtime Concepts

- **Artboard**: the visual component to render.
- **State machine**: interaction logic.
- **Input**: a value your app can control.
- **Binding**: a runtime-controlled property such as text or color.
- **Event**: a message emitted by the animation.

## Export Goal

Export should produce a small `.strut` file that can be shipped with your frontend app and controlled without opening Strut Studio.
