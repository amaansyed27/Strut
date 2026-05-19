# Generate A Character

Strut can start from a plain prompt or attached reference images and create an editable character animation.

This is useful when you want a product mascot, helper bot, empty-state character, onboarding guide, or small animated brand moment but do not have a finished mockup yet.

## Try It

Open Strut Studio, create or open a project, then use the chat composer.

To use a reference:

1. Click **Reference** in the composer.
2. Attach a PNG, JPEG, WebP, GIF, or SVG.
3. Add a short instruction such as "make this a friendly helper bot" or generate from the reference alone.

Example prompts:

```txt
make a minimalist waving robot like the reference image
make an owl mascot with wave, blink, scan and celebrate animations
make a scanner robot with a face scan animation
make a celebration robot with success and confetti animation
```

Press the generate button in the composer. Strut creates a `.strut` document with:

- an artboard
- named layers
- timelines
- a state machine
- runtime inputs
- events

The current pre-alpha built-in generator creates a small set of deterministic character families. When a BYOK or local vision-capable provider is selected in the desktop app, attached references are included in the generation request so the model can use the image composition, silhouette, pose, and palette. The workflow is the important contract: prompts and references produce editable Strut documents, not flat images.

## Review The Result

After generation, switch to **Chat + preview** to review the character in motion. Switch to **Editor** to inspect files, layers, parts, and state controls.

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

If you want to compare directions before building, ask Strut to plan first:

```txt
plan three directions for a friendly support mascot before building
```

Review the rough 2D directions, choose one such as Floating Helper, Scanner Bot, Celebration Bot, or Owl Guide, then ask Strut to build the full editable character.

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
