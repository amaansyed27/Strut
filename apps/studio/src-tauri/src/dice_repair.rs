use serde_json::Value;
use uuid::Uuid;

fn id_to_uuid(id: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes())
}

fn part_text(part: &Value) -> String {
    let id = part.get("id").and_then(Value::as_str).unwrap_or("");
    let name = part.get("name").and_then(Value::as_str).unwrap_or("");
    let role = part.get("role").and_then(Value::as_str).unwrap_or("");
    format!("{} {} {}", id, name, role).to_ascii_lowercase()
}

fn subject_is_dice(classification: &str, label: &str, name: &str) -> bool {
    let text = format!("{classification} {label} {name}").to_ascii_lowercase();
    text.contains("dice") || text.contains(" die") || text.contains("rolling die")
}

fn plan_string_values(plan: &Value, key: &str) -> Vec<String> {
    plan.get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(|value| value.to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

fn timeline_state_names(plan: &Value) -> Vec<String> {
    plan.get("timelines")
        .and_then(Value::as_array)
        .map(|timelines| {
            timelines
                .iter()
                .filter_map(|timeline| {
                    timeline
                        .get("state")
                        .or_else(|| timeline.get("id"))
                        .or_else(|| timeline.get("name"))
                        .and_then(Value::as_str)
                })
                .map(|value| value.to_ascii_lowercase())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn should_replace_with_canonical_dice(
    plan: &Value,
    classification: &str,
    label: &str,
    name: &str,
) -> bool {
    if !subject_is_dice(classification, label, name) {
        return false;
    }

    let states = plan_string_values(plan, "states");
    let uses_canonical_face_states = (1..=6).all(|face| {
        let state = format!("face_{face}");
        states.iter().any(|candidate| candidate == &state)
    });
    if !uses_canonical_face_states {
        return false;
    }

    let timeline_states = timeline_state_names(plan);
    let missing_face_timeline = (1..=6).any(|face| {
        let state = format!("face_{face}");
        !timeline_states.iter().any(|candidate| candidate == &state)
    });

    let parts = plan
        .get("parts")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let pip_parts = parts
        .iter()
        .filter(|part| {
            let text = part_text(part);
            text.contains("pip") || text.contains("dot")
        })
        .count();
    let face_pip_parts = parts
        .iter()
        .filter(|part| {
            let text = part_text(part);
            text.contains("face") && (text.contains("pip") || text.contains("dot"))
        })
        .count();

    missing_face_timeline || pip_parts < 6 || face_pip_parts < 6
}

fn dice_style(fill: Option<&str>, stroke: Option<&str>, stroke_width: f32, opacity: f32) -> strut_core::Style {
    strut_core::Style {
        fill: fill.map(str::to_string),
        stroke: stroke.map(str::to_string),
        stroke_width,
        opacity,
        linecap: Some("round".to_string()),
        linejoin: Some("round".to_string()),
    }
}

fn dice_transform(x: f32, y: f32) -> strut_core::Transform {
    let mut transform = strut_core::Transform::default();
    transform.translate_x = x;
    transform.translate_y = y;
    transform
}

fn dice_node(
    id: &str,
    name: &str,
    kind: strut_core::NodeKind,
    role: &str,
    shape: strut_core::Shape,
    transform: strut_core::Transform,
    style: strut_core::Style,
    children: Vec<strut_core::Node>,
) -> strut_core::Node {
    strut_core::Node {
        id: id_to_uuid(id),
        name: name.to_string(),
        kind,
        role: Some(role.to_string()),
        transform,
        style,
        shape,
        children,
    }
}

fn dice_track(target: &str, property: &str, frames: Vec<(u32, f32, strut_core::Easing)>) -> strut_core::Track {
    strut_core::Track {
        target: id_to_uuid(target),
        property: property.to_string(),
        keyframes: frames
            .into_iter()
            .map(|(time_ms, value, easing)| strut_core::Keyframe {
                time_ms,
                value: strut_core::PropertyValue::Number(value),
                easing,
            })
            .collect(),
    }
}

fn dice_pip_group(face: usize, positions: &[(f32, f32)], base_opacity: f32) -> strut_core::Node {
    let children = positions
        .iter()
        .enumerate()
        .map(|(index, (x, y))| {
            dice_node(
                &format!("Face{face}Pip{}", index + 1),
                &format!("Face {face} Pip {}", index + 1),
                strut_core::NodeKind::Ellipse,
                "pip",
                strut_core::Shape::Ellipse {
                    cx: 0.0,
                    cy: 0.0,
                    rx: 8.0,
                    ry: 8.0,
                },
                dice_transform(*x, *y),
                dice_style(Some("#111827"), None, 0.0, 1.0),
                vec![],
            )
        })
        .collect();

    dice_node(
        &format!("Face{face}Pips"),
        &format!("Face {face} Pips"),
        strut_core::NodeKind::Group,
        "pip face",
        strut_core::Shape::None,
        strut_core::Transform::default(),
        dice_style(None, None, 0.0, base_opacity),
        children,
    )
}

pub(crate) fn canonical_dice_document(name: &str) -> strut_core::Document {
    let face_positions: [(usize, &[(f32, f32)]); 6] = [
        (1, &[(0.0, 0.0)]),
        (2, &[(-36.0, -36.0), (36.0, 36.0)]),
        (3, &[(-36.0, -36.0), (0.0, 0.0), (36.0, 36.0)]),
        (4, &[(-36.0, -36.0), (36.0, -36.0), (-36.0, 36.0), (36.0, 36.0)]),
        (5, &[(-36.0, -36.0), (36.0, -36.0), (0.0, 0.0), (-36.0, 36.0), (36.0, 36.0)]),
        (6, &[(-36.0, -42.0), (36.0, -42.0), (-36.0, 0.0), (36.0, 0.0), (-36.0, 42.0), (36.0, 42.0)]),
    ];

    let pips = dice_node(
        "Pips",
        "Pips",
        strut_core::NodeKind::Group,
        "pip container",
        strut_core::Shape::None,
        strut_core::Transform::default(),
        dice_style(None, None, 0.0, 1.0),
        face_positions
            .iter()
            .map(|(face, positions)| dice_pip_group(*face, positions, if *face == 6 { 1.0 } else { 0.0 }))
            .collect(),
    );

    let die_group = dice_node(
        "DieGroup",
        "Die Group",
        strut_core::NodeKind::Group,
        "dice",
        strut_core::Shape::None,
        dice_transform(480.0, 260.0),
        dice_style(None, None, 0.0, 1.0),
        vec![
            dice_node(
                "DieBody",
                "DieBody",
                strut_core::NodeKind::Rect,
                "volume",
                strut_core::Shape::Rect { x: -66.0, y: -56.0, width: 144.0, height: 144.0, rx: 24.0 },
                dice_transform(14.0, 16.0),
                dice_style(Some("#d9e2ef"), Some("#111827"), 3.0, 1.0),
                vec![],
            ),
            dice_node(
                "BackFace",
                "Back Face",
                strut_core::NodeKind::Rect,
                "back face",
                strut_core::Shape::Rect { x: -70.0, y: -64.0, width: 144.0, height: 144.0, rx: 24.0 },
                dice_transform(8.0, 8.0),
                dice_style(Some("#eef3f8"), Some("#111827"), 3.0, 1.0),
                vec![],
            ),
            dice_node(
                "RightFace",
                "Right Face",
                strut_core::NodeKind::Path,
                "right face",
                strut_core::Shape::Path { d: "M72 -58 L112 -34 L112 72 L72 88 Z".to_string() },
                strut_core::Transform::default(),
                dice_style(Some("#d7e0eb"), Some("#111827"), 3.0, 1.0),
                vec![],
            ),
            dice_node(
                "TopFace",
                "Top Face",
                strut_core::NodeKind::Path,
                "top face",
                strut_core::Shape::Path { d: "M-72 -72 L-30 -104 L74 -80 L72 -58 Z".to_string() },
                strut_core::Transform::default(),
                dice_style(Some("#f8fafc"), Some("#111827"), 3.0, 1.0),
                vec![],
            ),
            dice_node(
                "FrontFace",
                "Front Face",
                strut_core::NodeKind::Rect,
                "front face",
                strut_core::Shape::Rect { x: -72.0, y: -72.0, width: 144.0, height: 144.0, rx: 24.0 },
                strut_core::Transform::default(),
                dice_style(Some("#ffffff"), Some("#111827"), 3.0, 1.0),
                vec![],
            ),
            dice_node(
                "FrontFaceAlias",
                "FrontFace",
                strut_core::NodeKind::Group,
                "compatibility alias",
                strut_core::Shape::None,
                strut_core::Transform::default(),
                dice_style(None, None, 0.0, 0.0),
                vec![],
            ),
            dice_node(
                "EdgeHighlight",
                "Edge Highlight",
                strut_core::NodeKind::Path,
                "edge highlight",
                strut_core::Shape::Path { d: "M-54 -62 L-20 -88 M78 -48 L98 -36 M-56 64 L64 64".to_string() },
                strut_core::Transform::default(),
                dice_style(None, Some("#c9d7e6"), 5.0, 0.78),
                vec![],
            ),
            pips,
        ],
    );

    let root = dice_node(
        "Root",
        "Root",
        strut_core::NodeKind::Group,
        "root",
        strut_core::Shape::None,
        strut_core::Transform::default(),
        dice_style(None, None, 0.0, 1.0),
        vec![
            dice_node(
                "SettleShadow",
                "SettleShadow",
                strut_core::NodeKind::Ellipse,
                "shadow",
                strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 92.0, ry: 15.0 },
                dice_transform(496.0, 388.0),
                dice_style(Some("#111827"), None, 0.0, 0.18),
                vec![],
            ),
            die_group,
        ],
    );

    let face_ids = ["Face1Pips", "Face2Pips", "Face3Pips", "Face4Pips", "Face5Pips", "Face6Pips"];
    let mut timelines = vec![
        strut_core::Timeline {
            id: id_to_uuid("dice-idle"),
            name: "idle".to_string(),
            duration_ms: 1600,
            loops: true,
            tracks: vec![
                dice_track("DieGroup", "translation.y", vec![(0, 0.0, strut_core::Easing::EaseInOut), (800, -8.0, strut_core::Easing::EaseInOut), (1600, 0.0, strut_core::Easing::EaseInOut)]),
                dice_track("DieGroup", "rotation", vec![(0, -2.0, strut_core::Easing::EaseInOut), (800, 2.0, strut_core::Easing::EaseInOut), (1600, -2.0, strut_core::Easing::EaseInOut)]),
                dice_track("SettleShadow", "scale.x", vec![(0, 1.0, strut_core::Easing::EaseInOut), (800, 0.86, strut_core::Easing::EaseInOut), (1600, 1.0, strut_core::Easing::EaseInOut)]),
            ],
        },
        strut_core::Timeline {
            id: id_to_uuid("dice-roll"),
            name: "roll".to_string(),
            duration_ms: 950,
            loops: true,
            tracks: vec![
                dice_track("DieGroup", "translation.y", vec![(0, 0.0, strut_core::Easing::EaseOut), (430, -58.0, strut_core::Easing::EaseOut), (950, 0.0, strut_core::Easing::EaseIn)]),
                dice_track("DieGroup", "rotation", vec![(0, 0.0, strut_core::Easing::Linear), (950, 720.0, strut_core::Easing::Linear)]),
                dice_track("DieGroup", "scale.x", vec![(0, 1.0, strut_core::Easing::Linear), (235, 0.18, strut_core::Easing::Linear), (470, -1.0, strut_core::Easing::Linear), (705, -0.18, strut_core::Easing::Linear), (950, 1.0, strut_core::Easing::Linear)]),
                dice_track("SettleShadow", "opacity", vec![(0, 0.18, strut_core::Easing::EaseOut), (430, 0.06, strut_core::Easing::EaseOut), (950, 0.18, strut_core::Easing::EaseIn)]),
            ],
        },
    ];

    for face in 1..=6 {
        let mut tracks = vec![
            dice_track("DieGroup", "translation.y", vec![(0, -24.0, strut_core::Easing::EaseOut), (360, 8.0, strut_core::Easing::EaseInOut), (700, 0.0, strut_core::Easing::EaseOut)]),
            dice_track("DieGroup", "scale", vec![(0, 0.92, strut_core::Easing::EaseOut), (360, 1.08, strut_core::Easing::EaseInOut), (700, 1.0, strut_core::Easing::EaseOut)]),
        ];
        for (index, target) in face_ids.iter().enumerate() {
            let visible = if index + 1 == face { 1.0 } else { 0.0 };
            tracks.push(dice_track(target, "opacity", vec![(0, visible, strut_core::Easing::Linear), (700, visible, strut_core::Easing::Linear)]));
        }
        timelines.push(strut_core::Timeline {
            id: id_to_uuid(&format!("dice-face-{face}")),
            name: format!("face_{face}"),
            duration_ms: 700,
            loops: false,
            tracks,
        });
    }

    strut_core::Document {
        id: id_to_uuid("canonical-dice-document"),
        name: if name.trim().is_empty() { "Premium 3D Dice Roller".to_string() } else { name.to_string() },
        artboards: vec![strut_core::Artboard {
            id: id_to_uuid("canonical-dice-artboard"),
            name: "Dice Artboard".to_string(),
            width: 960.0,
            height: 540.0,
            nodes: vec![root],
        }],
        timelines,
        state_machines: vec![strut_core::StateMachine {
            id: id_to_uuid("canonical-dice-controller"),
            name: "Controller".to_string(),
            states: vec![
                "idle".to_string(),
                "roll".to_string(),
                "face_1".to_string(),
                "face_2".to_string(),
                "face_3".to_string(),
                "face_4".to_string(),
                "face_5".to_string(),
                "face_6".to_string(),
            ],
            inputs: vec![],
            transitions: vec![],
        }],
        bindings: vec![],
        events: vec![],
    }
}
