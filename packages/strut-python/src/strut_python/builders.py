from __future__ import annotations

from .model import Binding, Ellipse, Path, Rect, Scene, State, Text, style
from .motion import attention_nudge, idle_breathe, progress_sweep, pulse, reveal, settle, soft_bob, tiny_tilt


def rolling_dice() -> Scene:
    sprites = [
        Rect("DieBody", "DieBody", "volume", 378, 174, 210, 210, 24, style=style("#f5f7fb", "#111827", 5)),
        Rect("FrontFace", "FrontFace", "front face", 402, 214, 168, 146, 16, style=style("#ffffff", "#111827", 4)),
        Path("TopFace", "TopFace", "top face", "M402 214 L454 168 L618 184 L570 214 Z", style=style("#e6edf7", "#111827", 4)),
        Path("Pips", "Pips", "number marks", "M442 252 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0 M530 320 m-8 0 a8 8 0 1 0 16 0 a8 8 0 1 0 -16 0", style=style(None, "#111827", 4)),
        Path("EdgeHighlight", "EdgeHighlight", "edge light", "M414 228 L454 188 L604 202", style=style(None, "#b7c7db", 7)),
        Ellipse("SettleShadow", "SettleShadow", "grounding shadow", 494, 414, 116, 18, style=style("#1f2937", None, 0, 0.22)),
    ]
    return Scene(
        id="sprite-python-dice-plan",
        name="Rolling Dice Motion",
        subject_classification="dice",
        subject_label="Rolling Dice",
        sprites=sprites,
        states=[State("idle"), State("settle")],
        timelines=[settle("DieBody")],
        motion_roles=[{"id": "primary", "purpose": "calm dice roll settle", "partRefs": ["DieBody", "FrontFace", "TopFace"]}],
        bindings=[Binding("edit_diebody_fill", "DieBody", "fill")],
        notes=["sprite-python dice builder; no mascot anatomy"],
    )


def abstract_logo_reveal() -> Scene:
    sprites = [
        Path("PrimaryMark", "PrimaryMark", "main vector mark", "M382 180 C450 120 540 146 582 222 C520 206 470 234 432 306 C398 266 370 226 382 180 Z", style=style("#6ee7b7", "#172033", 5)),
        Text("Wordmark", "Wordmark", "brand text", 396, 384, "STRUT", 42, style=style("#172033", None, 0)),
        Path("AccentStroke", "AccentStroke", "accent line", "M392 326 C452 352 528 348 596 312", style=style(None, "#2563eb", 8)),
        Rect("RevealMask", "RevealMask", "reveal mask", 360, 154, 280, 250, 20, style=style("#ffffff", None, 0, 0.08)),
        Path("AnchorGrid", "AnchorGrid", "alignment grid", "M360 270 L640 270 M500 150 L500 410", style=style(None, "#94a3b8", 2, 0.38)),
        Ellipse("Glow", "Glow", "soft emphasis", 498, 266, 118, 76, style=style("#dbeafe", None, 0, 0.28)),
    ]
    return Scene(
        id="sprite-python-logo-plan",
        name="Abstract Logo Motion",
        subject_classification="logo",
        subject_label="Abstract Logo",
        sprites=sprites,
        states=[State("idle"), State("reveal")],
        timelines=[reveal("PrimaryMark"), tiny_tilt("AccentStroke", "reveal", "accent-tilt")],
        motion_roles=[{"id": "primary", "purpose": "mark reveal without mascot anatomy", "partRefs": ["PrimaryMark", "AccentStroke"]}],
        bindings=[Binding("edit_primarymark_fill", "PrimaryMark", "fill")],
        notes=["sprite-python abstract logo builder"],
    )


def loader_progress() -> Scene:
    sprites = [
        Ellipse("Track", "Track", "background track", 480, 270, 120, 120, style=style(None, "#cbd5e1", 14)),
        Path("ActiveSegment", "ActiveSegment", "active arc", "M480 150 A120 120 0 0 1 600 270", style=style(None, "#14b8a6", 16)),
        Ellipse("PulseDot", "PulseDot", "pulse marker", 600, 270, 14, 14, style=style("#0f766e", "#0f766e", 2)),
        Path("ProgressSweep", "ProgressSweep", "sweep indicator", "M480 270 L600 270", style=style(None, "#99f6e4", 6)),
        Ellipse("Glow", "Glow", "soft glow", 480, 270, 144, 144, style=style("#ccfbf1", None, 0, 0.25)),
        Text("CenterLabel", "CenterLabel", "progress label", 454, 282, "42%", 24, style=style("#134e4a", None, 0)),
    ]
    return Scene(
        id="sprite-python-loader-plan",
        name="Progress Loader Motion",
        subject_classification="loader",
        subject_label="Progress Loader",
        sprites=sprites,
        states=[State("idle"), State("loading"), State("pulse")],
        timelines=[progress_sweep("ActiveSegment"), pulse("PulseDot")],
        motion_roles=[{"id": "primary", "purpose": "calm progress sweep", "partRefs": ["ActiveSegment", "PulseDot"]}],
        bindings=[Binding("edit_active_segment_stroke", "ActiveSegment", "stroke")],
        notes=["sprite-python loader builder; no face or body"],
    )


def mascot_idle() -> Scene:
    sprites = [
        Ellipse("Body", "Body", "body", 480, 306, 92, 118, style=style("#a7f3d0", "#064e3b", 6)),
        Ellipse("Head", "Head", "head", 480, 190, 82, 68, style=style("#ecfdf5", "#064e3b", 6)),
        Path("Eyes", "Eyes", "eyes", "M446 186 q10 -16 20 0 M494 186 q10 -16 20 0", style=style(None, "#064e3b", 8)),
        Path("Arms", "Arms", "arms", "M394 292 C350 310 344 352 382 364 M566 292 C610 310 616 352 578 364", style=style(None, "#047857", 10)),
        Ellipse("AccentBadge", "AccentBadge", "accent", 512, 316, 16, 16, style=style("#34d399", "#064e3b", 3)),
        Ellipse("GroundShadow", "GroundShadow", "shadow", 480, 438, 108, 16, style=style("#064e3b", None, 0, 0.2)),
    ]
    return Scene(
        id="sprite-python-mascot-plan",
        name="Helpful Mascot Motion",
        subject_classification="mascot",
        subject_label="Helpful Mascot",
        sprites=sprites,
        states=[State("idle"), State("hover")],
        timelines=[idle_breathe("Body"), soft_bob("Head"), attention_nudge("Arms")],
        motion_roles=[{"id": "primary", "purpose": "quiet mascot idle motion", "partRefs": ["Body", "Head", "Eyes"]}],
        bindings=[Binding("edit_body_fill", "Body", "fill")],
        notes=["sprite-python mascot builder; anatomy is present because subject is mascot"],
    )


def ui_microinteraction() -> Scene:
    sprites = [
        Rect("ButtonSurface", "ButtonSurface", "control surface", 370, 254, 220, 72, 12, style=style("#e0f2fe", "#0f172a", 4)),
        Text("ButtonLabel", "ButtonLabel", "label", 426, 297, "Continue", 22, style=style("#0f172a", None, 0)),
        Path("FocusRing", "FocusRing", "focus ring", "M364 248 L596 248 L596 332 L364 332 Z", style=style(None, "#38bdf8", 4, 0.65)),
        Path("CheckMark", "CheckMark", "success mark", "M444 294 L470 316 L520 264", style=style(None, "#16a34a", 7)),
        Ellipse("HoverGlow", "HoverGlow", "hover glow", 480, 290, 138, 52, style=style("#bae6fd", None, 0, 0.22)),
    ]
    return Scene(
        id="sprite-python-ui-plan",
        name="Button Microinteraction Motion",
        subject_classification="ui",
        subject_label="Button Microinteraction",
        sprites=sprites,
        states=[State("idle"), State("hover")],
        timelines=[attention_nudge("ButtonSurface")],
        motion_roles=[{"id": "primary", "purpose": "small responsive UI nudge", "partRefs": ["ButtonSurface", "FocusRing"]}],
        bindings=[Binding("edit_button_fill", "ButtonSurface", "fill")],
        notes=["sprite-python UI microinteraction builder"],
    )

