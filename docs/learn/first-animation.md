# Create Your First Animation

The first Strut workflow is a small interactive login button. It is intentionally simple because it touches the whole product:

- artwork
- layers
- timelines
- state machines
- runtime inputs
- verifier checks
- export

## The Goal

Create a button with these states:

```txt
idle
hover
pressed
loading
success
error
```

The button should be controllable by app code. For example, a React app should be able to set loading, fire success, or listen for an event.

## How Strut Thinks About It

A Strut animation is not just a timeline. It is a component with named controls.

```txt
Artboard: LoginButton
Layers: ButtonSurface, Label, SpinnerArc, SuccessCheck
State machine: Interaction
Inputs: hover, pressed, status
Events: submit, completed
```

## Current Status

The sample login button is currently a scaffold used to prove the Studio shell, Rust core, and validation path. The next milestones will turn it into a real `.strut` sample file that Studio loads and validates.
