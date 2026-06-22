use uuid::Uuid;

fn id_to_uuid(id: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes())
}

pub(crate) fn is_coin_prompt(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    lower.contains("coin") && (lower.contains("flip") || lower.contains("2.5d") || lower.contains("front") || lower.contains("back"))
}

fn coin_style(fill: Option<&str>, stroke: Option<&str>, stroke_width: f32, opacity: f32) -> strut_core::Style {
    strut_core::Style {
        fill: fill.map(str::to_string),
        stroke: stroke.map(str::to_string),
        stroke_width,
        opacity,
        linecap: Some("round".to_string()),
        linejoin: Some("round".to_string()),
    }
}

fn coin_transform(x: f32, y: f32) -> strut_core::Transform {
    let mut transform = strut_core::Transform::default();
    transform.translate_x = x;
    transform.translate_y = y;
    transform
}

fn coin_node(
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

fn coin_track(target: &str, property: &str, frames: Vec<(u32, f32, strut_core::Easing)>) -> strut_core::Track {
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

pub(crate) fn canonical_coin_document(name: &str) -> strut_core::Document {
    let coin_group = coin_node(
        "CoinGroup",
        "Coin Group",
        strut_core::NodeKind::Group,
        "coin",
        strut_core::Shape::None,
        coin_transform(480.0, 242.0),
        coin_style(None, None, 0.0, 1.0),
        vec![
            coin_node("BackFace", "Back Face", strut_core::NodeKind::Ellipse, "back face", strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 112.0, ry: 112.0 }, strut_core::Transform::default(), coin_style(Some("#f0a53a"), Some("#7a4a0f"), 5.0, 0.0), vec![]),
            coin_node("RimDepth", "Rim Depth", strut_core::NodeKind::Ellipse, "rim depth", strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 119.0, ry: 112.0 }, coin_transform(10.0, 8.0), coin_style(Some("#b56a16"), Some("#6b3f09"), 6.0, 0.86), vec![]),
            coin_node("FrontFace", "Front Face", strut_core::NodeKind::Ellipse, "front face", strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 112.0, ry: 112.0 }, strut_core::Transform::default(), coin_style(Some("#ffd76a"), Some("#8a550f"), 5.0, 1.0), vec![]),
            coin_node("InnerRing", "Inner Ring", strut_core::NodeKind::Ellipse, "engraved ring", strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 78.0, ry: 78.0 }, strut_core::Transform::default(), coin_style(None, Some("#fff4b8"), 6.0, 0.68), vec![]),
            coin_node("Medallion", "Medallion", strut_core::NodeKind::Ellipse, "premium inner medallion", strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 52.0, ry: 52.0 }, strut_core::Transform::default(), coin_style(Some("#f2bd3f"), Some("#9f6416"), 4.0, 0.92), vec![]),
            coin_node("FrontMark", "Heads Mark", strut_core::NodeKind::Text, "heads emblem", strut_core::Shape::Text { x: -26.0, y: 28.0, value: "H".to_string(), size: 82.0 }, strut_core::Transform::default(), coin_style(Some("#7a4a0f"), None, 0.0, 0.78), vec![]),
            coin_node("BackMark", "Tails Mark", strut_core::NodeKind::Text, "tails emblem", strut_core::Shape::Text { x: -22.0, y: 28.0, value: "T".to_string(), size: 82.0 }, strut_core::Transform::default(), coin_style(Some("#7a4a0f"), None, 0.0, 0.0), vec![]),
            coin_node("GlintA", "Glint A", strut_core::NodeKind::Path, "glint", strut_core::Shape::Path { d: "M-78 -58 C-36 -88 34 -88 78 -44".to_string() }, strut_core::Transform::default(), coin_style(None, Some("#fff8cc"), 9.0, 0.76), vec![]),
            coin_node("GlintB", "Glint B", strut_core::NodeKind::Path, "glint", strut_core::Shape::Path { d: "M-52 70 C-10 90 42 78 70 42".to_string() }, strut_core::Transform::default(), coin_style(None, Some("#fff2a8"), 6.0, 0.38), vec![]),
        ],
    );

    let root = coin_node(
        "Root",
        "Root",
        strut_core::NodeKind::Group,
        "root",
        strut_core::Shape::None,
        strut_core::Transform::default(),
        coin_style(None, None, 0.0, 1.0),
        vec![
            coin_node("ReactiveShadow", "Reactive Shadow", strut_core::NodeKind::Ellipse, "reactive shadow", strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx: 132.0, ry: 20.0 }, coin_transform(492.0, 392.0), coin_style(Some("#0f172a"), None, 0.0, 0.22), vec![]),
            coin_group,
        ],
    );

    let timelines = vec![
        strut_core::Timeline {
            id: id_to_uuid("coin-idle"),
            name: "idle".to_string(),
            duration_ms: 1600,
            loops: true,
            tracks: vec![
                coin_track("CoinGroup", "translation.y", vec![(0, 0.0, strut_core::Easing::EaseInOut), (800, -8.0, strut_core::Easing::EaseInOut), (1600, 0.0, strut_core::Easing::EaseInOut)]),
                coin_track("GlintA", "opacity", vec![(0, 0.28, strut_core::Easing::EaseInOut), (800, 0.92, strut_core::Easing::EaseInOut), (1600, 0.28, strut_core::Easing::EaseInOut)]),
                coin_track("ReactiveShadow", "scale.x", vec![(0, 1.0, strut_core::Easing::EaseInOut), (800, 0.84, strut_core::Easing::EaseInOut), (1600, 1.0, strut_core::Easing::EaseInOut)]),
            ],
        },
        strut_core::Timeline {
            id: id_to_uuid("coin-anticipation"),
            name: "anticipation".to_string(),
            duration_ms: 460,
            loops: false,
            tracks: vec![
                coin_track("CoinGroup", "scale", vec![(0, 1.0, strut_core::Easing::EaseOut), (260, 0.86, strut_core::Easing::EaseInOut), (460, 1.08, strut_core::Easing::EaseOut)]),
                coin_track("CoinGroup", "translation.y", vec![(0, 0.0, strut_core::Easing::EaseOut), (260, 18.0, strut_core::Easing::EaseInOut), (460, -22.0, strut_core::Easing::EaseOut)]),
                coin_track("ReactiveShadow", "scale.x", vec![(0, 1.0, strut_core::Easing::EaseOut), (260, 1.35, strut_core::Easing::EaseInOut), (460, 0.68, strut_core::Easing::EaseOut)]),
            ],
        },
        strut_core::Timeline {
            id: id_to_uuid("coin-flip"),
            name: "flip".to_string(),
            duration_ms: 1200,
            loops: true,
            tracks: vec![
                coin_track("CoinGroup", "translation.y", vec![(0, -22.0, strut_core::Easing::EaseOut), (360, -112.0, strut_core::Easing::EaseOut), (820, -74.0, strut_core::Easing::EaseInOut), (1200, 0.0, strut_core::Easing::EaseIn)]),
                coin_track("CoinGroup", "scale.x", vec![(0, 1.0, strut_core::Easing::Linear), (150, 0.18, strut_core::Easing::Linear), (300, -1.0, strut_core::Easing::Linear), (450, -0.18, strut_core::Easing::Linear), (600, 1.0, strut_core::Easing::Linear), (900, -1.0, strut_core::Easing::Linear), (1200, 1.0, strut_core::Easing::Linear)]),
                coin_track("CoinGroup", "rotation", vec![(0, -8.0, strut_core::Easing::Linear), (1200, 720.0, strut_core::Easing::Linear)]),
                coin_track("FrontFace", "opacity", vec![(0, 1.0, strut_core::Easing::Linear), (300, 0.0, strut_core::Easing::Linear), (600, 1.0, strut_core::Easing::Linear), (900, 0.0, strut_core::Easing::Linear), (1200, 1.0, strut_core::Easing::Linear)]),
                coin_track("BackFace", "opacity", vec![(0, 0.0, strut_core::Easing::Linear), (300, 1.0, strut_core::Easing::Linear), (600, 0.0, strut_core::Easing::Linear), (900, 1.0, strut_core::Easing::Linear), (1200, 0.0, strut_core::Easing::Linear)]),
                coin_track("FrontMark", "opacity", vec![(0, 0.78, strut_core::Easing::Linear), (300, 0.0, strut_core::Easing::Linear), (600, 0.78, strut_core::Easing::Linear), (900, 0.0, strut_core::Easing::Linear), (1200, 0.78, strut_core::Easing::Linear)]),
                coin_track("BackMark", "opacity", vec![(0, 0.0, strut_core::Easing::Linear), (300, 0.78, strut_core::Easing::Linear), (600, 0.0, strut_core::Easing::Linear), (900, 0.78, strut_core::Easing::Linear), (1200, 0.0, strut_core::Easing::Linear)]),
                coin_track("ReactiveShadow", "opacity", vec![(0, 0.22, strut_core::Easing::EaseOut), (360, 0.06, strut_core::Easing::EaseOut), (1200, 0.22, strut_core::Easing::EaseIn)]),
            ],
        },
        strut_core::Timeline {
            id: id_to_uuid("coin-settle"),
            name: "settle".to_string(),
            duration_ms: 760,
            loops: false,
            tracks: vec![
                coin_track("CoinGroup", "translation.y", vec![(0, -40.0, strut_core::Easing::EaseOut), (340, 10.0, strut_core::Easing::EaseInOut), (760, 0.0, strut_core::Easing::EaseOut)]),
                coin_track("CoinGroup", "scale", vec![(0, 0.94, strut_core::Easing::EaseOut), (340, 1.08, strut_core::Easing::EaseInOut), (760, 1.0, strut_core::Easing::EaseOut)]),
                coin_track("CoinGroup", "rotation", vec![(0, 24.0, strut_core::Easing::EaseOut), (520, -4.0, strut_core::Easing::EaseInOut), (760, 0.0, strut_core::Easing::EaseOut)]),
                coin_track("ReactiveShadow", "scale.x", vec![(0, 0.7, strut_core::Easing::EaseOut), (340, 1.26, strut_core::Easing::EaseInOut), (760, 1.0, strut_core::Easing::EaseOut)]),
            ],
        },
        strut_core::Timeline { id: id_to_uuid("coin-heads"), name: "heads".to_string(), duration_ms: 500, loops: false, tracks: vec![coin_track("FrontFace", "opacity", vec![(0, 1.0, strut_core::Easing::Linear), (500, 1.0, strut_core::Easing::Linear)]), coin_track("BackFace", "opacity", vec![(0, 0.0, strut_core::Easing::Linear), (500, 0.0, strut_core::Easing::Linear)]), coin_track("FrontMark", "opacity", vec![(0, 0.78, strut_core::Easing::Linear), (500, 0.78, strut_core::Easing::Linear)]), coin_track("BackMark", "opacity", vec![(0, 0.0, strut_core::Easing::Linear), (500, 0.0, strut_core::Easing::Linear)])] },
        strut_core::Timeline { id: id_to_uuid("coin-tails"), name: "tails".to_string(), duration_ms: 500, loops: false, tracks: vec![coin_track("FrontFace", "opacity", vec![(0, 0.0, strut_core::Easing::Linear), (500, 0.0, strut_core::Easing::Linear)]), coin_track("BackFace", "opacity", vec![(0, 1.0, strut_core::Easing::Linear), (500, 1.0, strut_core::Easing::Linear)]), coin_track("FrontMark", "opacity", vec![(0, 0.0, strut_core::Easing::Linear), (500, 0.0, strut_core::Easing::Linear)]), coin_track("BackMark", "opacity", vec![(0, 0.78, strut_core::Easing::Linear), (500, 0.78, strut_core::Easing::Linear)])] },
    ];

    strut_core::Document {
        id: id_to_uuid("canonical-coin-document"),
        name: if name.trim().is_empty() { "Premium 2.5D Coin Flip".to_string() } else { name.to_string() },
        artboards: vec![strut_core::Artboard { id: id_to_uuid("canonical-coin-artboard"), name: "Coin Flip Artboard".to_string(), width: 960.0, height: 540.0, nodes: vec![root] }],
        timelines,
        state_machines: vec![strut_core::StateMachine {
            id: id_to_uuid("canonical-coin-controller"),
            name: "Controller".to_string(),
            states: vec!["idle".to_string(), "anticipation".to_string(), "flip".to_string(), "settle".to_string(), "heads".to_string(), "tails".to_string()],
            inputs: vec![],
            transitions: vec![],
        }],
        bindings: vec![],
        events: vec![],
    }
}
