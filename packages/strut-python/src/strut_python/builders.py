from __future__ import annotations

import re

from .model import Binding, Ellipse, Path, Rect, Scene, State, Text, style
from .motion import attention_nudge, blink, glance, idle_breathe, progress_sweep, pulse, reveal, settle, soft_bob, tiny_tilt, wing_wave


STOPWORDS = {
    "a",
    "an",
    "and",
    "animate",
    "animation",
    "asset",
    "calm",
    "cinematic",
    "create",
    "for",
    "gentle",
    "icon",
    "idle",
    "in",
    "into",
    "make",
    "motion",
    "of",
    "soft",
    "static",
    "style",
    "the",
    "with",
}


def _words(instruction: str) -> list[str]:
    return [word for word in re.findall(r"[a-zA-Z][a-zA-Z0-9]*", instruction.lower()) if word not in STOPWORDS]


def _pascal(words: list[str], fallback: str = "DynamicAsset") -> str:
    clean = [word[:18] for word in words if word]
    if not clean:
        return fallback
    return "".join(word.capitalize() for word in clean[:3])


def _label(words: list[str], fallback: str = "Dynamic Asset") -> str:
    if not words:
        return fallback
    return " ".join(word.capitalize() for word in words[:4])


def procedural_asset(instruction: str) -> Scene:
    """Build a subject-aware sprite plan when no named fixture fits.

    This is intentionally deterministic so coding agents can reproduce and patch
    it, but the part names and motion roles come from the prompt instead of a
    fixed mascot/body template.
    """

    lower = instruction.lower()
    words = _words(instruction)
    base = _pascal(words)
    label = _label(words)
    wants_heavy = any(token in lower for token in ["lively", "immersive", "cinematic", "character", "companion", "sprite", "story"])

    if "bird" in lower or "twitter" in lower:
        sprites = [
            Path(f"{base}Body", f"{label} Body", "primary silhouette", "M376 282 C430 202 540 202 596 274 C540 262 502 286 470 350 C430 330 396 308 376 282 Z", style=style("#7dd3fc", "#0f172a", 5)),
            Path(f"{base}Wing", f"{label} Wing", "lift wing", "M450 280 C496 220 574 218 626 262 C560 266 518 304 486 354 Z", style=style("#38bdf8", "#075985", 4)),
            Path(f"{base}Tail", f"{label} Tail", "tail feathers", "M388 286 L328 248 L354 318 Z", style=style("#bae6fd", "#075985", 4)),
            Path(f"{base}Beak", f"{label} Beak", "direction point", "M596 274 L646 254 L606 300 Z", style=style("#fbbf24", "#92400e", 3)),
            Path(f"{base}Trail", f"{label} Motion Trail", "flight trail", "M320 342 C382 376 514 386 638 340", style=style(None, "#a7f3d0", 6, 0.5)),
            Ellipse(f"{base}Shadow", f"{label} Shadow", "soft shadow", 480, 424, 132, 16, style=style("#0f172a", None, 0, 0.18)),
        ]
        classification = "bird_icon"
        timelines = [
            soft_bob(f"{base}Body", "idle", f"{base.lower()}-flight-bob"),
            tiny_tilt(f"{base}Wing", "idle", f"{base.lower()}-wing-lift"),
            reveal(f"{base}Trail", f"{base.lower()}-trail-reveal"),
        ]
        motion_roles = [
            {"id": "primary", "purpose": "subject-specific flight silhouette", "partRefs": [f"{base}Body", f"{base}Wing"]},
            {"id": "energy", "purpose": "motion trail and lift without mascot-only anatomy", "partRefs": [f"{base}Trail", f"{base}Shadow"]},
        ]
    else:
        detail_parts: list[Path] = []
        if "lava" in lower or "volcano" in lower:
            detail_parts.append(
                Path(
                    f"{base}Lava",
                    f"{label} Lava Flow",
                    "emissive lava shimmer",
                    "M424 286 C456 254 484 300 512 270 C538 246 558 284 586 258",
                    style=style(None, "#f97316", 8, 0.86),
                )
            )
        if "smoke" in lower or "mist" in lower or "orbit" in lower:
            detail_parts.append(
                Path(
                    f"{base}Smoke",
                    f"{label} Smoke Orbit",
                    "smoke orbit",
                    "M350 246 C404 166 548 150 636 226 C586 218 536 228 486 260 C438 292 394 296 350 246 Z",
                    style=style(None, "#94a3b8", 6, 0.42),
                )
            )
        if not detail_parts:
            detail_parts.append(
                Path(
                    f"{base}Accent",
                    f"{label} Accent Stroke",
                    "editable accent",
                    "M386 336 C450 372 548 370 612 320",
                    style=style(None, "#2563eb", 8),
                )
            )
            detail_parts.append(
                Path(
                    f"{base}Trail",
                    f"{label} Motion Trail",
                    "motion arc",
                    "M344 274 C394 206 560 166 646 230",
                    style=style(None, "#99f6e4", 6, 0.45),
                )
            )
        sprites = [
            Path(f"{base}Core", f"{label} Core", "primary form", "M392 202 C452 144 552 158 602 238 C574 332 470 376 390 326 C354 282 356 236 392 202 Z", style=style("#d9f99d" if wants_heavy else "#e0f2fe", "#13231b", 5)),
            Path(f"{base}Facet", f"{label} Facet", "secondary plane", "M430 214 C482 190 542 206 564 252 C520 246 480 270 448 314 C430 278 420 244 430 214 Z", style=style("#a7f3d0", "#047857", 4, 0.9)),
            *detail_parts,
            Path(f"{base}Spark", f"{label} Spark", "small signal", "M596 174 L608 206 L642 214 L610 226 L598 258 L586 226 L552 216 L584 206 Z", style=style("#fde68a", "#92400e", 3)),
            Ellipse(f"{base}Shadow", f"{label} Shadow", "grounding shadow", 490, 420, 130, 18, style=style("#13231b", None, 0, 0.16)),
        ]
        classification = "dynamic_asset"
        timelines = [
            reveal(f"{base}Core", f"{base.lower()}-core-reveal"),
            tiny_tilt(detail_parts[0].id, "reveal", f"{base.lower()}-detail-drift"),
            attention_nudge(f"{base}Spark", f"{base.lower()}-spark-nudge"),
        ]
        if wants_heavy:
            timelines.append(soft_bob(f"{base}Core", "idle", f"{base.lower()}-ambient-bob"))
        motion_roles = [
            {"id": "primary", "purpose": f"subject-aware motion for {label}", "partRefs": [f"{base}Core", f"{base}Facet"]},
            {"id": "detail", "purpose": "editable prompt-specific details, sparkle, and motion trail", "partRefs": [part.id for part in detail_parts] + [f"{base}Spark"]},
        ]

    return Scene(
        id=f"sprite-python-{base.lower()}-plan",
        name=f"{label} Motion",
        subject_classification=classification,
        subject_label=label,
        sprites=sprites,
        states=[State("idle"), State("reveal"), State("focus"), State("hover")],
        timelines=timelines,
        motion_roles=motion_roles,
        bindings=[Binding(f"edit_{base.lower()}_fill", sprites[0].id, "fill")],
        notes=[f"sprite-python procedural builder from instruction: {instruction[:140]}"],
    )


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
        Ellipse("Body", "Body", "soft body volume", 480, 312, 94, 118, style=style("#a7f3d0", "#064e3b", 6)),
        Ellipse("BellyPatch", "BellyPatch", "warm belly patch", 480, 332, 58, 76, style=style("#ecfdf5", "#047857", 4, 0.92)),
        Ellipse("Head", "Head", "head", 480, 190, 86, 70, style=style("#ecfdf5", "#064e3b", 6)),
        Path("FaceMask", "FaceMask", "face mask", "M420 188 C438 148 522 148 540 188 C526 220 434 220 420 188 Z", style=style("#d1fae5", "#047857", 3)),
        Ellipse("LeftEye", "LeftEye", "left eye", 452, 188, 10, 13, style=style("#064e3b", "#064e3b", 2)),
        Ellipse("RightEye", "RightEye", "right eye", 508, 188, 10, 13, style=style("#064e3b", "#064e3b", 2)),
        Path("EyeGlints", "EyeGlints", "eye glints", "M448 184 m-2 0 a2 2 0 1 0 4 0 a2 2 0 1 0 -4 0 M504 184 m-2 0 a2 2 0 1 0 4 0 a2 2 0 1 0 -4 0", style=style(None, "#ffffff", 2)),
        Path("BeakSmile", "BeakSmile", "tiny beak smile", "M470 206 L480 216 L490 206 M456 224 Q480 238 504 224", style=style(None, "#92400e", 5)),
        Path("LeftWing", "LeftWing", "left wing", "M394 292 C350 310 344 352 382 364 C412 356 422 324 410 296 Z", style=style("#6ee7b7", "#047857", 5)),
        Path("RightWing", "RightWing", "right wing", "M566 292 C610 310 616 352 578 364 C548 356 538 324 550 296 Z", style=style("#6ee7b7", "#047857", 5)),
        Path("Feet", "Feet", "small feet", "M436 430 q22 16 44 0 M480 430 q22 16 44 0", style=style(None, "#92400e", 7)),
        Ellipse("AccentBadge", "AccentBadge", "agent status badge", 512, 316, 16, 16, style=style("#34d399", "#064e3b", 3)),
        Path("AmbientHalo", "AmbientHalo", "low energy halo", "M386 246 C412 118 552 118 584 246", style=style(None, "#8be9fd", 5, 0.42)),
        Ellipse("GroundShadow", "GroundShadow", "soft grounding shadow", 480, 438, 118, 16, style=style("#064e3b", None, 0, 0.2)),
    ]
    return Scene(
        id="sprite-python-mascot-plan",
        name="Companion Mascot Motion",
        subject_classification="mascot",
        subject_label="Companion Mascot",
        sprites=sprites,
        states=[State("idle"), State("hover"), State("blink"), State("focus"), State("wave")],
        timelines=[
            idle_breathe("Body"),
            soft_bob("Head"),
            blink("LeftEye", "left-soft-blink", "left_soft_blink"),
            blink("RightEye", "right-soft-blink", "right_soft_blink"),
            glance("FaceMask"),
            wing_wave("RightWing"),
            attention_nudge("AccentBadge"),
        ],
        motion_roles=[
            {"id": "primary", "purpose": "quiet companion idle motion with soft breathing", "partRefs": ["Body", "Head", "BellyPatch"]},
            {"id": "attention", "purpose": "subtle blink and glance that feels alive without distraction", "partRefs": ["LeftEye", "RightEye", "FaceMask"]},
            {"id": "greeting", "purpose": "gentle wing wave for low-energy acknowledgement", "partRefs": ["RightWing", "AmbientHalo"]},
        ],
        bindings=[Binding("edit_body_fill", "Body", "fill")],
        notes=["sprite-python companion mascot builder; anatomy is present because subject is mascot"],
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


def icon_badge() -> Scene:
    sprites = [
        Ellipse("BadgePlate", "BadgePlate", "badge base", 480, 270, 106, 106, style=style("#eef2ff", "#1e1b4b", 5)),
        Path("InnerShield", "InnerShield", "inner shield", "M480 188 L552 220 L540 308 C526 350 504 374 480 386 C456 374 434 350 420 308 L408 220 Z", style=style("#c7d2fe", "#312e81", 4)),
        Path("SparkGlyph", "SparkGlyph", "spark glyph", "M480 222 L494 260 L534 270 L494 280 L480 318 L466 280 L426 270 L466 260 Z", style=style("#fef3c7", "#92400e", 4)),
        Path("OrbitStroke", "OrbitStroke", "orbit stroke", "M386 286 C430 224 534 214 584 260", style=style(None, "#38bdf8", 7, 0.82)),
        Ellipse("StatusDot", "StatusDot", "status dot", 566, 206, 18, 18, style=style("#22c55e", "#14532d", 3)),
        Text("BadgeLabel", "BadgeLabel", "short label", 434, 410, "VERIFIED", 24, style=style("#1e1b4b", None, 0)),
    ]
    return Scene(
        id="sprite-python-icon-badge-plan",
        name="Icon Badge Motion",
        subject_classification="badge",
        subject_label="Icon Badge",
        sprites=sprites,
        states=[State("idle"), State("reveal"), State("success")],
        timelines=[reveal("InnerShield", "badge-reveal"), pulse("StatusDot", "success", "status-pulse")],
        motion_roles=[{"id": "primary", "purpose": "badge reveal and calm success pulse", "partRefs": ["BadgePlate", "InnerShield", "StatusDot"]}],
        bindings=[Binding("edit_badge_fill", "BadgePlate", "fill")],
        notes=["sprite-python icon badge builder; no mascot anatomy"],
    )
