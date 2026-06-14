# Motion Language

Strut's default motion style is quiet, low-energy, and useful inside product interfaces. It should feel alive without demanding attention.

This is inspired by small companion animations: subtle breathing, tiny bobs, soft pauses, restrained acknowledgement, and readable state changes. It is not a requirement to use pet sprites, mascot art, or any fixed atlas format. The same language can apply to a logo, button, loader, icon, badge, empty state, chart accent, or character.

## Default Feel

- Calm loops over dramatic gestures.
- Small position changes over large jumps.
- Short, readable states over cinematic scenes.
- Tiny tilt, blink, scale, opacity, or scan changes.
- Pauses that make the animation feel intentional.
- Reduced-motion compatibility from the idle state.

## State Semantics

The early generated documents use a compact reusable state set:

```txt
idle       still, low-distraction baseline
float      gentle breathing or drift
wave       small acknowledgement
blink      tiny reset or attention pulse
scan       focused inspection or processing
celebrate  restrained success
sleep      reduced attention or rest
```

These names are not mascot-only. For a button, `wave` can be a hover acknowledgement. For a loader, `scan` can be a small progress sweep. For a logo, `celebrate` can be a soft settle after reveal.

## Avoid By Default

- Giant jumps.
- Fast shaking.
- Speed lines.
- Confetti storms.
- Camera moves.
- Heavy squash and stretch.
- Motion that pulls attention away from the main app task.

Use bigger motion only when the user explicitly asks for it.
