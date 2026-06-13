from __future__ import annotations

from .model import Keyframe, Timeline, track


def idle_breathe(target: str, timeline_id: str = "idle-breathe") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="idle_breathe",
        state="idle",
        duration_ms=1400,
        tracks=[
            track(target, "scale.y", [Keyframe(0, 1.0), Keyframe(700, 1.035, "ease_out"), Keyframe(1400, 1.0)]),
            track(target, "translation.y", [Keyframe(0, 0.0), Keyframe(700, -4.0, "ease_out"), Keyframe(1400, 0.0)]),
        ],
    )


def soft_bob(target: str, state: str = "idle", timeline_id: str = "soft-bob") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="soft_bob",
        state=state,
        duration_ms=1200,
        tracks=[track(target, "translation.y", [Keyframe(0, 0.0), Keyframe(600, -8.0, "ease_out"), Keyframe(1200, 0.0)])],
    )


def tiny_tilt(target: str, state: str = "idle", timeline_id: str = "tiny-tilt") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="tiny_tilt",
        state=state,
        duration_ms=1200,
        tracks=[track(target, "rotation", [Keyframe(0, -2.0), Keyframe(600, 2.0, "ease_out"), Keyframe(1200, -2.0)])],
    )


def settle(target: str, timeline_id: str = "settle") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="settle",
        state="settle",
        duration_ms=900,
        tracks=[
            track(target, "translation.y", [Keyframe(0, -18.0, "ease_out"), Keyframe(520, 4.0, "ease_in_out"), Keyframe(900, 0.0)]),
            track(target, "rotation", [Keyframe(0, -9.0, "ease_out"), Keyframe(520, 2.0, "ease_in_out"), Keyframe(900, 0.0)]),
        ],
    )


def reveal(target: str, timeline_id: str = "reveal") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="reveal",
        state="reveal",
        duration_ms=1000,
        tracks=[
            track(target, "opacity", [Keyframe(0, 0.15, "ease_out"), Keyframe(420, 1.0, "ease_out"), Keyframe(1000, 1.0)]),
            track(target, "translation.y", [Keyframe(0, 18.0, "ease_out"), Keyframe(640, 0.0, "ease_out"), Keyframe(1000, 0.0)]),
        ],
    )


def pulse(target: str, state: str = "pulse", timeline_id: str = "pulse") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="pulse",
        state=state,
        duration_ms=960,
        tracks=[track(target, "scale", [Keyframe(0, 0.92), Keyframe(480, 1.08, "ease_out"), Keyframe(960, 0.92)])],
    )


def progress_sweep(target: str, timeline_id: str = "progress-sweep") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="progress_sweep",
        state="loading",
        duration_ms=1200,
        tracks=[track(target, "rotation", [Keyframe(0, 0.0, "linear"), Keyframe(1200, 360.0, "linear")])],
    )


def attention_nudge(target: str, timeline_id: str = "attention-nudge") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="attention_nudge",
        state="hover",
        duration_ms=520,
        tracks=[track(target, "translation.x", [Keyframe(0, 0.0), Keyframe(170, 5.0, "ease_out"), Keyframe(340, -2.0), Keyframe(520, 0.0)])],
    )


def blink(target: str, timeline_id: str = "soft-blink", timeline_name: str = "soft_blink") -> Timeline:
    return Timeline(
        id=timeline_id,
        name=timeline_name,
        state="blink",
        duration_ms=360,
        tracks=[track(target, "scale.y", [Keyframe(0, 1.0), Keyframe(150, 0.18, "ease_out"), Keyframe(360, 1.0)])],
    )


def glance(target: str, timeline_id: str = "curious-glance") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="curious_glance",
        state="focus",
        duration_ms=1100,
        tracks=[
            track(target, "translation.x", [Keyframe(0, 0.0), Keyframe(460, 7.0, "ease_out"), Keyframe(1100, 0.0)]),
            track(target, "translation.y", [Keyframe(0, 0.0), Keyframe(460, -2.0, "ease_out"), Keyframe(1100, 0.0)]),
        ],
    )


def wing_wave(target: str, timeline_id: str = "gentle-wave") -> Timeline:
    return Timeline(
        id=timeline_id,
        name="gentle_wave",
        state="wave",
        duration_ms=980,
        tracks=[
            track(target, "rotation", [Keyframe(0, -3.0), Keyframe(280, 7.0, "ease_out"), Keyframe(620, -5.0), Keyframe(980, 0.0)]),
            track(target, "translation.y", [Keyframe(0, 0.0), Keyframe(280, -5.0, "ease_out"), Keyframe(980, 0.0)]),
        ],
    )
