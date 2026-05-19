# Create Your First Animation

The first Strut workflows are a small interactive login button and a minimalist animated bot. They are intentionally simple because they touch the whole product:

- artwork
- layers
- timelines
- state machines
- runtime inputs
- verifier checks
- export

## Login Button Goal

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

## Bot Goal

Create a bot like a lightweight product assistant with these states:

```txt
idle
float
wave
blink
scan
celebrate
sleep
```

The bot proves Strut can represent character-style product motion with a state machine, multiple timelines, named layers, runtime bindings, and events.

## How Strut Thinks About It

A Strut animation is not just a timeline. It is a component with named controls.

```txt
Artboard: LoginButton
Layers: ButtonSurface, Label, SpinnerArc, SuccessCheck
State machine: Interaction
Inputs: hover, pressed, status
Events: submit, completed
```

```txt
Artboard: MinimalBot
Layers: BotRig, HelmetShell, FacePanel, Eyes, Smile, Torso, RightArm
State machine: BotMoods
Inputs: mode, wave, scan
Events: wave_started, celebration_complete
```

## Current Status

The sample login button is a real `.strut` file used to prove the Studio shell, Rust core, and validation path.

You can validate it from the repository root:

```powershell
cargo run -p strut-format --example validate -- samples/login-button.strut
cargo run -p strut-format --example validate -- samples/minimal-bot.strut
```
