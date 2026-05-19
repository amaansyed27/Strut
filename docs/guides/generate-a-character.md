# Generate A Character

Strut can start from a plain prompt and create an editable character animation.

This is useful when you want a product mascot, helper bot, empty-state character, onboarding guide, or small animated brand moment but do not have a finished mockup yet.

## Try It

Open Strut Studio and use the prompt box in Plan Mode.

Example prompts:

```txt
make a minimalist waving robot like the reference image
make an owl mascot with wave, blink, scan and celebrate animations
make a scanner robot with a face scan animation
make a celebration robot with success and confetti animation
```

Click **Generate Character**. Strut creates a `.strut` document with:

- an artboard
- named layers
- timelines
- a state machine
- runtime inputs
- events

The current pre-alpha generator creates a small set of deterministic character families. The workflow is the important contract: prompts produce editable Strut documents, not flat images.

## Review The Result

After generation, check the left layer list and the state controls.

For a bot, you should see layers such as:

```txt
BotRig
HelmetShell
FacePanel
Eyes
Smile
Torso
RightArm
```

For an owl, you should see layers such as:

```txt
OwlRig
OwlBody
FaceMask
Beak
LeftWing
RightWing
```

Use the state buttons to preview motion states:

```txt
idle
float
wave
blink
scan
celebrate
sleep
```

## Sketch First

If you want to compare directions before building, click **Generate Character** and review the Plan Mode sketches. Choose a direction such as Floating Helper, Scanner Bot, Celebration Bot, or Owl Guide, then click **Build Character**.

This keeps generation controllable. You decide the concept before Strut builds the full scene graph and motion controls.

## What Is Editable

The generated document is a normal Strut file. The character is represented as structured parts:

- shapes
- groups
- layer names
- style data
- timelines
- state machine inputs
- events

That means future Strut tools can edit the character, retime motion, rename layers, bind states to app code, and export to runtimes.

## Current Limits

Strut is pre-alpha. Character generation currently uses a local deterministic generator so the workflow can be tested without API keys.

The intended AI-first path is:

```txt
prompt or mockup
  -> plan
  -> structured character spec
  -> editable scene graph
  -> timelines and state machine
  -> verifier checks
  -> runtime export
```

BYOK model providers and local agents will plug into this pipeline instead of replacing it with one-off images.
