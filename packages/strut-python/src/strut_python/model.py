from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Iterable


EDITABLE_PROPERTIES = ["fill", "translation.x", "translation.y", "rotation", "opacity"]


def style(fill: str | None = "#f6f0df", stroke: str | None = "#25221d", stroke_width: float = 5.0, opacity: float = 1.0) -> dict[str, Any]:
    return {
        "fill": fill,
        "stroke": stroke,
        "strokeWidth": stroke_width,
        "opacity": opacity,
    }


@dataclass(frozen=True)
class Keyframe:
    time_ms: int
    value: float
    easing: str = "ease_in_out"

    def to_plan(self) -> dict[str, Any]:
        return {"timeMs": self.time_ms, "value": self.value, "easing": self.easing}

    def to_operation(self, timeline: str, target: str, property_name: str) -> dict[str, Any]:
        return {
            "type": "add_keyframe",
            "timeline": timeline,
            "target": target,
            "property": property_name,
            "time_ms": self.time_ms,
            "value": self.value,
            "easing": self.easing,
        }


@dataclass(frozen=True)
class Timeline:
    id: str
    name: str
    state: str
    duration_ms: int
    tracks: list[dict[str, Any]]

    def to_plan(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "name": self.name,
            "state": self.state,
            "durationMs": self.duration_ms,
            "tracks": [
                {
                    "target": track["target"],
                    "property": track["property"],
                    "keyframes": [keyframe.to_plan() for keyframe in track["keyframes"]],
                }
                for track in self.tracks
            ],
        }

    def to_operations(self) -> list[dict[str, Any]]:
        operations = [
            {
                "type": "add_timeline",
                "id": self.id,
                "name": self.name,
                "state": self.state,
                "duration_ms": self.duration_ms,
            }
        ]
        for track in self.tracks:
            for keyframe in track["keyframes"]:
                operations.append(keyframe.to_operation(self.id, track["target"], track["property"]))
        return operations


@dataclass(frozen=True)
class Binding:
    name: str
    target: str
    property: str

    def to_operation(self) -> dict[str, Any]:
        return {
            "type": "bind_property",
            "name": self.name,
            "target": self.target,
            "property": self.property,
        }


@dataclass(frozen=True)
class State:
    name: str

    def to_operation(self) -> dict[str, Any]:
        return {"type": "add_state", "state": self.name}


@dataclass(frozen=True)
class Sprite:
    id: str
    name: str
    role: str
    geometry: dict[str, Any]
    style: dict[str, Any] = field(default_factory=style)
    motion_roles: tuple[str, ...] = ("primary",)
    editable: bool = True
    allowed_properties: tuple[str, ...] = tuple(EDITABLE_PROPERTIES)

    @property
    def kind(self) -> str:
        return self.geometry["kind"]

    def to_part(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "name": self.name,
            "role": self.role,
            "geometry": self.geometry,
            "style": self.style,
            "motionRoles": list(self.motion_roles),
            "constraints": {
                "editable": self.editable,
                "allowedProperties": list(self.allowed_properties),
            },
        }

    def to_create_operation(self, parent: str) -> dict[str, Any]:
        return {
            "type": "create_node",
            "id": self.id,
            "name": self.name,
            "kind": self.kind,
            "parent": parent,
            "geometry": self.geometry,
            "style": self.style,
            "role": self.role,
        }


@dataclass(frozen=True)
class Rect(Sprite):
    def __init__(self, id: str, name: str, role: str, x: float, y: float, width: float, height: float, rx: float = 0.0, **kwargs: Any) -> None:
        super().__init__(id, name, role, {"kind": "rect", "x": x, "y": y, "width": width, "height": height, "rx": rx}, **kwargs)


@dataclass(frozen=True)
class Ellipse(Sprite):
    def __init__(self, id: str, name: str, role: str, cx: float, cy: float, rx: float, ry: float, **kwargs: Any) -> None:
        super().__init__(id, name, role, {"kind": "ellipse", "cx": cx, "cy": cy, "rx": rx, "ry": ry}, **kwargs)


@dataclass(frozen=True)
class Path(Sprite):
    def __init__(self, id: str, name: str, role: str, d: str, **kwargs: Any) -> None:
        super().__init__(id, name, role, {"kind": "path", "d": d}, **kwargs)


@dataclass(frozen=True)
class Text(Sprite):
    def __init__(self, id: str, name: str, role: str, x: float, y: float, value: str, size: float, **kwargs: Any) -> None:
        super().__init__(id, name, role, {"kind": "text", "x": x, "y": y, "value": value, "size": size}, **kwargs)


@dataclass(frozen=True)
class Group:
    id: str
    name: str
    children: tuple[str, ...]

    def to_operation(self) -> dict[str, Any]:
        return {"type": "group_nodes", "id": self.id, "name": self.name, "children": list(self.children)}


@dataclass
class Scene:
    id: str
    name: str
    subject_classification: str
    subject_label: str
    sprites: list[Sprite]
    states: list[State]
    timelines: list[Timeline]
    motion_roles: list[dict[str, Any]]
    bindings: list[Binding] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    def to_envelope(self) -> dict[str, Any]:
        group = Group("SceneRig", f"{self.name} Rig", tuple(sprite.id for sprite in self.sprites))
        editable_parts = [sprite.id for sprite in self.sprites if sprite.editable]
        operations = [group.to_operation()]
        operations.extend(sprite.to_create_operation(group.id) for sprite in self.sprites)
        operations.extend(state.to_operation() for state in self.states)
        for timeline in self.timelines:
            operations.extend(timeline.to_operations())
        operations.extend(binding.to_operation() for binding in self.bindings)
        operations.append(
            {
                "type": "emit_event",
                "name": "sprite_python_plan_emitted",
                "description": f"{self.subject_label} authored by sprite-python and awaiting Rust validation",
            }
        )

        return {
            "plan": {
                "id": self.id,
                "name": self.name,
                "subject": {"classification": self.subject_classification, "label": self.subject_label},
                "parts": [sprite.to_part() for sprite in self.sprites],
                "motionRoles": self.motion_roles,
                "states": [state.name for state in self.states],
                "timelines": [timeline.to_plan() for timeline in self.timelines],
                "editability": {
                    "editableParts": editable_parts,
                    "lockedParts": [],
                    "notes": self.notes,
                },
            },
            "operations": operations,
        }


def track(target: str, property_name: str, keyframes: Iterable[Keyframe]) -> dict[str, Any]:
    return {"target": target, "property": property_name, "keyframes": list(keyframes)}
