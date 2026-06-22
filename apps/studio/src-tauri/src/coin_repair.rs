use uuid::Uuid;

fn id_to_uuid(id: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes())
}

fn style(fill: Option<&str>, stroke: Option<&str>, stroke_width: f32, opacity: f32) -> strut_core::Style {
    strut_core::Style {
        fill: fill.map(str::to_string),
        stroke: stroke.map(str::to_string),
        stroke_width,
        opacity,
        linecap: Some("round".to_string()),
        linejoin: Some("round".to_string()),
    }
}

fn transform(x: f32, y: f32) -> strut_core::Transform {
    let mut transform = strut_core::Transform::default();
    transform.translate_x = x;
    transform.translate_y = y;
    transform
}

fn node(
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

fn track(target: &str, property: &str, frames: Vec<(u32, f32, strut_core::Easing)>) -> strut_core::Track {
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

pub(crate) fn prompt_is_coin_like(prompt: &str) -> bool {
    let text = prompt.to_ascii_lowercase();
    text.contains("coin")
        || text.contains("medallion")
        || text.contains("medal")
        || text.contains("heads")
        || text.contains("tails")
}

pub(crate) fn canonical_coin_document(name: &str) -> strut_core::Document {
    let front_face = node(
        "CoinFrontFace",
        "Front Face Group",
        strut_core::NodeKind::Group,
        "front face",
        strut_core::Shape::None,
        strut_core::Transform::default(),
        style(None, None, 0.0, 1.0),
        vec![
            node(
                "FrontFaceSurface",
                "Front Face Surface",
                strut_core::NodeKind::Ellipse,
                "front face surface",
                strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 72.0, ry: 72.0 },
                strut_core::Transform::default(),
                style(Some("#f8c70a"), Some("#5f3b00"), 4.0, 1.0),
                vec![],
            ),
            node(
                "FrontOuterRim",
                "Front Outer Rim",
                strut_core::NodeKind::Ellipse,
                "rim",
                strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 64.0, ry: 64.0 },
                strut_core::Transform::default(),
                style(None, Some("#fff1a8"), 7.0, 0.92),
                vec![],
            ),
            node(
                "FrontInnerBezel",
                "Front Inner Bezel",
                strut_core::NodeKind::Ellipse,
                "inner bezel",
                strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 48.0, ry: 48.0 },
                strut_core::Transform::default(),
                style(None, Some("#ad7609"), 4.0, 0.9),
                vec![],
            ),
            node(
                "FrontEmblem",
                "Front Emblem Star",
                strut_core::NodeKind::Path,
                "front emblem detail",
                strut_core::Shape::Path { d: "M0 -34 L10 -10 L36 -10 L15 5 L24 32 L0 16 L-24 32 L-15 5 L-36 -10 L-10 -10 Z".to_string() },
                strut_core::Transform::default(),
                style(Some("#7c4a00"), Some("#fff0a6"), 3.0, 0.95),
                vec![],
            ),
        ],
    );

    let back_face = node(
        "CoinBackFace",
        "Back Face Group",
        strut_core::NodeKind::Group,
        "back face",
        strut_core::Shape::None,
        strut_core::Transform::default(),
        style(None, None, 0.0, 0.0),
        vec![
            node(
                "BackFaceSurface",
                "Back Face Surface",
                strut_core::NodeKind::Ellipse,
                "back face surface",
                strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 72.0, ry: 72.0 },
                strut_core::Transform::default(),
                style(Some("#e39a08"), Some("#5f3b00"), 4.0, 1.0),
                vec![],
            ),
            node(
                "BackOuterRim",
                "Back Outer Rim",
                strut_core::NodeKind::Ellipse,
                "back rim",
                strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 64.0, ry: 64.0 },
                strut_core::Transform::default(),
                style(None, Some("#ffe29a"), 7.0, 0.9),
                vec![],
            ),
            node(
                "BackEmblem",
                "Back Emblem Orbit",
                strut_core::NodeKind::Path,
                "back emblem detail",
                strut_core::Shape::Path { d: "M-38 0 C-20 -26 20 -26 38 0 C20 26 -20 26 -38 0 M0 -38 C26 -20 26 20 0 38 C-26 20 -26 -20 0 -38".to_string() },
                strut_core::Transform::default(),
                style(None, Some("#7b4c05"), 5.0, 0.95),
                vec![],
            ),
        ],
    );

    let coin_rig = node(
        "CoinRig",
        "Coin Rig",
        strut_core::NodeKind::Group,
        "coin rig",
        strut_core::Shape::None,
        transform(480.0, 258.0),
        style(None, None, 0.0, 1.0),
        vec![
            node(
                "RimDepthBack",
                "Rim Depth Back Plate",
                strut_core::NodeKind::Ellipse,
                "rim depth edge",
                strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 78.0, ry: 73.0 },
                transform(9.0, 10.0),
                style(Some("#7c4a06"), Some("#3b2300"), 3.0, 1.0),
                vec![],
            ),
            node(
                "RimDepthSide",
                "Warm Side Thickness",
                strut_core::NodeKind::Ellipse,
                "side edge depth",
                strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 76.0, ry: 72.0 },
                transform(5.0, 6.0),
                style(Some("#b97809"), Some("#5b3500"), 2.0, 1.0),
                vec![],
            ),
            front_face,
            back_face,
            node(
                "MovingGlint",
                "Moving Glint Highlight",
                strut_core::NodeKind::Path,
                "glint highlight polish",
                strut_core::Shape::Path { d: "M-42 -40 C-18 -58 18 -58 42 -40".to_string() },
                strut_core::Transform::default(),
                style(None, Some("#fff8d2"), 6.0, 0.82),
                vec![],
            ),
            node(
                "SettleSpark",
                "Settle Spark",
                strut_core::NodeKind::Path,
                "spark polish",
                strut_core::Shape::Path { d: "M86 -54 L86 -22 M70 -38 L102 -38".to_string() },
                strut_core::Transform::default(),
                style(None, Some("#fff7bf"), 5.0, 0.0),
                vec![],
            ),
        ],
    );

    let root = node(
        "Root",
        "Root",
        strut_core::NodeKind::Group,
        "root",
        strut_core::Shape::None,
        strut_core::Transform::default(),
        style(None, None, 0.0, 1.0),
        vec![
            node(
                "ReactiveGroundShadow",
                "Reactive Ground Shadow",
                strut_core::NodeKind::Ellipse,
                "reactive ground shadow",
                strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 96.0, ry: 17.0 },
                transform(488.0, 392.0),
                style(Some("#1f2937"), None, 0.0, 0.18),
                vec![],
            ),
            coin_rig,
        ],
    );

    let timelines = vec![
        strut_core::Timeline {
            id: id_to_uuid("coin-idle"),
            name: "idle".to_string(),
            duration_ms: 1600,
            loops: true,
            tracks: vec![
                track("CoinRig", "translation.y", vec![(0, 0.0, strut_core::Easing::EaseInOut), (800, -8.0, strut_core::Easing::EaseInOut), (1600, 0.0, strut_core::Easing::EaseInOut)]),
                track("CoinRig", "rotation", vec![(0, -2.0, strut_core::Easing::EaseInOut), (800, 2.0, strut_core::Easing::EaseInOut), (1600, -2.0, strut_core::Easing::EaseInOut)]),
                track("ReactiveGroundShadow", "scale.x", vec![(0, 1.0, strut_core::Easing::EaseInOut), (800, 0.86, strut_core::Easing::EaseInOut), (1600, 1.0, strut_core::Easing::EaseInOut)]),
            ],
        },
        strut_core::Timeline {
            id: id_to_uuid("coin-anticipation"),
            name: "anticipation".to_string(),
            duration_ms: 520,
            loops: false,
            tracks: vec![
                track("CoinRig", "translation.y", vec![(0, 0.0, strut_core::Easing::EaseOut), (220, 10.0, strut_core::Easing::EaseIn), (520, -18.0, strut_core::Easing::EaseOut)]),
                track("CoinRig", "scale.x", vec![(0, 1.0, strut_core::Easing::EaseOut), (220, 1.12, strut_core::Easing::EaseIn), (520, 0.92, strut_core::Easing::EaseOut)]),
                track("ReactiveGroundShadow", "scale.x", vec![(0, 1.0, strut_core::Easing::EaseOut), (220, 1.18, strut_core::Easing::EaseIn), (520, 0.74, strut_core::Easing::EaseOut)]),
            ],
        },
        strut_core::Timeline {
            id: id_to_uuid("coin-flip"),
            name: "flip".to_string(),
            duration_ms: 1250,
            loops: false,
            tracks: vec![
                track("CoinRig", "translation.y", vec![(0, -16.0, strut_core::Easing::EaseOut), (420, -96.0, strut_core::Easing::EaseOut), (900, -20.0, strut_core::Easing::EaseInOut), (1250, 0.0, strut_core::Easing::EaseOut)]),
                track("CoinRig", "scale.x", vec![(0, 1.0, strut_core::Easing::EaseInOut), (260, 0.14, strut_core::Easing::EaseInOut), (520, -1.0, strut_core::Easing::EaseInOut), (780, -0.16, strut_core::Easing::EaseInOut), (1020, 1.05, strut_core::Easing::EaseOut), (1250, 1.0, strut_core::Easing::EaseOut)]),
                track("CoinRig", "rotation", vec![(0, 0.0, strut_core::Easing::EaseInOut), (520, 210.0, strut_core::Easing::Linear), (1250, 360.0, strut_core::Easing::EaseOut)]),
                track("CoinFrontFace", "opacity", vec![(0, 1.0, strut_core::Easing::Linear), (510, 1.0, strut_core::Easing::Linear), (540, 0.0, strut_core::Easing::Linear), (1250, 0.0, strut_core::Easing::Linear)]),
                track("CoinBackFace", "opacity", vec![(0, 0.0, strut_core::Easing::Linear), (510, 0.0, strut_core::Easing::Linear), (540, 1.0, strut_core::Easing::Linear), (1250, 1.0, strut_core::Easing::Linear)]),
                track("MovingGlint", "translation.x", vec![(0, -42.0, strut_core::Easing::EaseOut), (680, 42.0, strut_core::Easing::EaseInOut), (1250, 8.0, strut_core::Easing::EaseOut)]),
                track("ReactiveGroundShadow", "opacity", vec![(0, 0.18, strut_core::Easing::EaseOut), (420, 0.04, strut_core::Easing::EaseOut), (1250, 0.20, strut_core::Easing::EaseOut)]),
                track("ReactiveGroundShadow", "scale.x", vec![(0, 0.76, strut_core::Easing::EaseOut), (420, 0.46, strut_core::Easing::EaseOut), (1250, 1.08, strut_core::Easing::EaseOut)]),
            ],
        },
        strut_core::Timeline {
            id: id_to_uuid("coin-settle"),
            name: "settle".to_string(),
            duration_ms: 720,
            loops: false,
            tracks: vec![
                track("CoinRig", "translation.y", vec![(0, -24.0, strut_core::Easing::EaseOut), (280, 8.0, strut_core::Easing::EaseInOut), (720, 0.0, strut_core::Easing::EaseOut)]),
                track("CoinRig", "scale", vec![(0, 0.94, strut_core::Easing::EaseOut), (280, 1.08, strut_core::Easing::EaseInOut), (720, 1.0, strut_core::Easing::EaseOut)]),
                track("CoinFrontFace", "opacity", vec![(0, 0.0, strut_core::Easing::Linear), (720, 0.0, strut_core::Easing::Linear)]),
                track("CoinBackFace", "opacity", vec![(0, 1.0, strut_core::Easing::Linear), (720, 1.0, strut_core::Easing::Linear)]),
                track("SettleSpark", "opacity", vec![(0, 0.0, strut_core::Easing::Linear), (160, 1.0, strut_core::Easing::EaseOut), (720, 0.0, strut_core::Easing::EaseIn)]),
                track("ReactiveGroundShadow", "scale.x", vec![(0, 0.82, strut_core::Easing::EaseOut), (280, 1.18, strut_core::Easing::EaseInOut), (720, 1.0, strut_core::Easing::EaseOut)]),
            ],
        },
    ];

    strut_core::Document {
        id: id_to_uuid("canonical-premium-coin-document"),
        name: if name.trim().is_empty() { "Premium 2.5D Coin Flip".to_string() } else { name.to_string() },
        artboards: vec![strut_core::Artboard {
            id: id_to_uuid("canonical-premium-coin-artboard"),
            name: "Coin Artboard".to_string(),
            width: 960.0,
            height: 540.0,
            nodes: vec![root],
        }],
        timelines,
        state_machines: vec![strut_core::StateMachine {
            id: id_to_uuid("canonical-premium-coin-controller"),
            name: "Controller".to_string(),
            states: vec!["idle".to_string(), "anticipation".to_string(), "flip".to_string(), "settle".to_string()],
            inputs: vec![],
            transitions: vec![],
        }],
        bindings: vec![],
        events: vec![],
    }
}
