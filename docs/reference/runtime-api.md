# Runtime API

The runtime API is how apps control exported Strut animations.

This API is not implemented yet. It is the intended developer experience for the MVP.

## JavaScript Runtime

```ts
const animation = await Strut.load("/login-button.strut", {
  canvas,
  artboard: "LoginButton",
  stateMachine: "Interaction",
});

animation.input("hover").set(true);
animation.input("pressed").fire();
animation.input("status").set("loading");

animation.on("completed", (event) => {
  console.log(event.name);
});
```

## React Runtime

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
