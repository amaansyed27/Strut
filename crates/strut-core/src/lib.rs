use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub artboards: Vec<Artboard>,
    pub timelines: Vec<Timeline>,
    pub state_machines: Vec<StateMachine>,
    pub bindings: Vec<Binding>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artboard {
    pub id: Uuid,
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub kind: NodeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default)]
    pub transform: Transform,
    #[serde(default)]
    pub style: Style,
    #[serde(default)]
    pub shape: Shape,
    #[serde(default)]
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Group,
    Rect,
    Ellipse,
    Path,
    Text,
    Image,
    HitArea,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    pub translate_x: f32,
    pub translate_y: f32,
    pub rotate: f32,
    #[serde(default)]
    pub rotate_x: f32,
    #[serde(default)]
    pub rotate_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            rotate: 0.0,
            rotate_x: 0.0,
            rotate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Style {
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f32,
    pub opacity: f32,
    pub linecap: Option<String>,
    pub linejoin: Option<String>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: None,
            stroke: None,
            stroke_width: 0.0,
            opacity: 1.0,
            linecap: None,
            linejoin: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Shape {
    #[default]
    None,
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rx: f32,
    },
    Ellipse {
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
    },
    Path {
        d: String,
    },
    Text {
        x: f32,
        y: f32,
        value: String,
        size: f32,
    },
    Sprite {
        url: String,
        frame_width: f32,
        frame_height: f32,
        columns: u32,
        rows: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterSpec {
    pub variant: String,
    pub name: Option<String>,
    pub accent: Option<String>,
    pub shell: Option<String>,
}

impl Default for CharacterSpec {
    fn default() -> Self {
        Self {
            variant: "floating-helper".to_string(),
            name: Some("Minimal Bot".to_string()),
            accent: Some("#51bfd0".to_string()),
            shell: Some("#f6f1e8".to_string()),
        }
    }
}

impl Node {
    pub fn new(id: u128, name: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id: Uuid::from_u128(id),
            name: name.into(),
            kind,
            role: None,
            transform: Transform::default(),
            style: Style::default(),
            shape: Shape::None,
            children: Vec::new(),
        }
    }

    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }
}

fn style(fill: Option<&str>, stroke: Option<&str>, stroke_width: f32) -> Style {
    Style {
        fill: fill.map(str::to_string),
        stroke: stroke.map(str::to_string),
        stroke_width,
        opacity: 1.0,
        linecap: Some("round".to_string()),
        linejoin: Some("round".to_string()),
    }
}

fn style_with_opacity(
    fill: Option<&str>,
    stroke: Option<&str>,
    stroke_width: f32,
    opacity: f32,
) -> Style {
    Style {
        opacity,
        ..style(fill, stroke, stroke_width)
    }
}

fn bot_nodes(shell: &str, accent: &str) -> Vec<Node> {
    vec![
        Node::new(103, "GroundShadow", NodeKind::Ellipse)
            .with_shape(Shape::Ellipse {
                cx: 480.0,
                cy: 442.0,
                rx: 116.0,
                ry: 18.0,
            })
            .with_style(style(Some("#17142f"), None, 0.0)),
        Node::new(102, "BotRig", NodeKind::Group).with_children(vec![
            Node::new(104, "HelmetShell", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M330 154 C348 70 434 38 544 58 C628 74 662 142 642 224 C620 312 540 350 432 328 C354 312 312 236 330 154Z".to_string(),
                })
                .with_style(style(Some(shell), Some("#17142f"), 8.0)),
            Node::new(105, "FacePanel", NodeKind::Rect)
                .with_shape(Shape::Rect {
                    x: 386.0,
                    y: 118.0,
                    width: 214.0,
                    height: 136.0,
                    rx: 42.0,
                })
                .with_style(style(Some("#17142f"), Some("#ffffff"), 6.0)),
            Node::new(115, "ScanSweep", NodeKind::Rect)
                .with_shape(Shape::Rect {
                    x: 404.0,
                    y: 170.0,
                    width: 178.0,
                    height: 10.0,
                    rx: 5.0,
                })
                .with_style(style_with_opacity(Some(accent), None, 0.0, 0.0)),
            Node::new(106, "Eyes", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M430 176 C438 154 462 154 470 176 M518 174 C526 152 550 152 558 174".to_string(),
                })
                .with_style(style(None, Some(accent), 9.0)),
            Node::new(107, "Smile", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M464 210 C480 230 512 230 528 208".to_string(),
                })
                .with_style(style(None, Some(accent), 9.0)),
            Node::new(108, "Torso", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M372 274 C386 226 430 204 492 206 C558 208 602 236 612 288 C624 356 576 402 486 400 C398 398 350 346 372 274Z".to_string(),
                })
                .with_style(style(Some(shell), Some("#17142f"), 8.0)),
            Node::new(109, "ChestLight", NodeKind::Ellipse)
                .with_shape(Shape::Ellipse {
                    cx: 512.0,
                    cy: 308.0,
                    rx: 17.0,
                    ry: 17.0,
                })
                .with_style(style(Some(accent), Some("#17142f"), 5.0)),
            Node::new(110, "LeftArm", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M332 304 C268 320 252 372 282 398 C310 424 352 388 382 344".to_string(),
                })
                .with_style(style(Some(shell), Some("#17142f"), 8.0)),
            Node::new(111, "RightArm", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M624 286 C686 296 716 252 690 218 C666 186 622 214 596 260".to_string(),
                })
                .with_style(style(Some(shell), Some("#17142f"), 8.0)),
            Node::new(112, "LeftLeg", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M420 376 C390 410 392 448 426 454 C458 460 476 424 482 388".to_string(),
                })
                .with_style(style(Some(shell), Some("#17142f"), 8.0)),
            Node::new(113, "RightLeg", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M532 382 C552 424 584 448 612 424 C636 402 610 366 570 344".to_string(),
                })
                .with_style(style(Some(shell), Some("#17142f"), 8.0)),
            Node::new(114, "Antennae", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M376 172 L346 112 M584 166 L616 86".to_string(),
                })
                .with_style(style(None, Some("#17142f"), 6.0)),
        ]),
    ]
}

fn owl_nodes(shell: &str, accent: &str) -> Vec<Node> {
    vec![
        Node::new(103, "GroundShadow", NodeKind::Ellipse)
            .with_shape(Shape::Ellipse {
                cx: 480.0,
                cy: 444.0,
                rx: 126.0,
                ry: 18.0,
            })
            .with_style(style(Some("#17331f"), None, 0.0)),
        Node::new(102, "OwlRig", NodeKind::Group).with_children(vec![
            Node::new(104, "OwlBody", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M340 178 C350 92 424 62 480 96 C536 60 612 94 624 182 C642 310 582 406 480 410 C378 406 322 306 340 178Z".to_string(),
                })
                .with_style(style(Some(shell), Some("#17331f"), 8.0)),
            Node::new(105, "FaceMask", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M384 184 C394 134 446 122 480 156 C514 122 566 134 576 184 C586 244 538 286 480 262 C422 286 374 244 384 184Z".to_string(),
                })
                .with_style(style(Some("#f6f1e8"), Some("#17331f"), 7.0)),
            Node::new(115, "ScanSweep", NodeKind::Rect)
                .with_shape(Shape::Rect {
                    x: 402.0,
                    y: 194.0,
                    width: 156.0,
                    height: 9.0,
                    rx: 5.0,
                })
                .with_style(style_with_opacity(Some(accent), None, 0.0, 0.0)),
            Node::new(106, "Eyes", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M424 196 C432 174 458 174 466 196 M494 196 C502 174 528 174 536 196".to_string(),
                })
                .with_style(style(None, Some("#17331f"), 10.0)),
            Node::new(107, "Beak", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M472 220 L488 220 L480 236Z".to_string(),
                })
                .with_style(style(Some("#f6d365"), Some("#17331f"), 4.0)),
            Node::new(108, "Belly", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M420 282 C430 342 530 342 540 282 C520 304 444 304 420 282Z".to_string(),
                })
                .with_style(style(Some("#d7f7c6"), Some("#17331f"), 5.0)),
            Node::new(109, "ChestMark", NodeKind::Ellipse)
                .with_shape(Shape::Ellipse {
                    cx: 480.0,
                    cy: 304.0,
                    rx: 18.0,
                    ry: 14.0,
                })
                .with_style(style(Some(accent), Some("#17331f"), 4.0)),
            Node::new(110, "LeftWing", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M368 248 C304 270 298 346 354 370 C382 332 390 292 368 248Z".to_string(),
                })
                .with_style(style(Some("#65c83e"), Some("#17331f"), 8.0)),
            Node::new(111, "RightWing", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M592 248 C656 270 662 346 606 370 C578 332 570 292 592 248Z".to_string(),
                })
                .with_style(style(Some("#65c83e"), Some("#17331f"), 8.0)),
            Node::new(112, "LeftFoot", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M430 404 C418 428 444 438 462 414".to_string(),
                })
                .with_style(style(None, Some("#f6d365"), 8.0)),
            Node::new(113, "RightFoot", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M530 404 C542 428 516 438 498 414".to_string(),
                })
                .with_style(style(None, Some("#f6d365"), 8.0)),
            Node::new(114, "BrowTufts", NodeKind::Path)
                .with_shape(Shape::Path {
                    d: "M412 138 L382 108 M548 138 L578 108".to_string(),
                })
                .with_style(style(None, Some("#17331f"), 8.0)),
        ]),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateMachine {
    pub id: Uuid,
    pub name: String,
    pub inputs: Vec<Input>,
    pub states: Vec<String>,
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Input {
    pub name: String,
    pub kind: InputKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    Boolean,
    Number,
    Trigger,
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub on: String,
    pub timeline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Timeline {
    pub id: Uuid,
    pub name: String,
    pub duration_ms: u32,
    #[serde(default)]
    pub loops: bool,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub target: Uuid,
    pub property: String,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Keyframe {
    pub time_ms: u32,
    pub value: PropertyValue,
    pub easing: Easing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum PropertyValue {
    Number(f32),
    Text(String),
    Color(String),
    Point { x: f32, y: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Binding {
    pub name: String,
    pub target: Uuid,
    pub property: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub name: String,
    pub description: String,
}

impl Document {
    pub fn sample_login_button() -> Self {
        Self {
            id: Uuid::nil(),
            name: "Login Button".to_string(),
            artboards: vec![Artboard {
                id: Uuid::from_u128(1),
                name: "LoginButton".to_string(),
                width: 960.0,
                height: 600.0,
                nodes: vec![
                    Node::new(2, "ButtonSurface", NodeKind::Rect).with_shape(Shape::Rect {
                        x: 370.0,
                        y: 268.0,
                        width: 220.0,
                        height: 64.0,
                        rx: 8.0,
                    }),
                    Node::new(3, "Label", NodeKind::Text).with_shape(Shape::Text {
                        x: 480.0,
                        y: 309.0,
                        value: "Sign in".to_string(),
                        size: 18.0,
                    }),
                    Node::new(4, "SpinnerArc", NodeKind::Path).with_shape(Shape::Path {
                        d: "M462 300 A18 18 0 1 1 461 299".to_string(),
                    }),
                ],
            }],
            timelines: vec![
                Timeline {
                    loops: false,
                    id: Uuid::from_u128(6),
                    name: "hover".to_string(),
                    duration_ms: 180,
                    tracks: vec![Track {
                        target: Uuid::from_u128(2),
                        property: "scale".to_string(),
                        keyframes: vec![
                            Keyframe {
                                time_ms: 0,
                                value: PropertyValue::Number(1.0),
                                easing: Easing::EaseOut,
                            },
                            Keyframe {
                                time_ms: 180,
                                value: PropertyValue::Number(1.04),
                                easing: Easing::EaseOut,
                            },
                        ],
                    }],
                },
                Timeline {
                    loops: false,
                    id: Uuid::from_u128(7),
                    name: "loading".to_string(),
                    duration_ms: 900,
                    tracks: vec![Track {
                        target: Uuid::from_u128(4),
                        property: "rotation".to_string(),
                        keyframes: vec![
                            Keyframe {
                                time_ms: 0,
                                value: PropertyValue::Number(0.0),
                                easing: Easing::Linear,
                            },
                            Keyframe {
                                time_ms: 900,
                                value: PropertyValue::Number(360.0),
                                easing: Easing::Linear,
                            },
                        ],
                    }],
                },
            ],
            state_machines: vec![StateMachine {
                id: Uuid::from_u128(5),
                name: "Interaction".to_string(),
                inputs: vec![
                    Input {
                        name: "hover".to_string(),
                        kind: InputKind::Boolean,
                    },
                    Input {
                        name: "pressed".to_string(),
                        kind: InputKind::Trigger,
                    },
                    Input {
                        name: "status".to_string(),
                        kind: InputKind::Enum,
                    },
                ],
                states: vec![
                    "idle".to_string(),
                    "hover".to_string(),
                    "pressed".to_string(),
                    "loading".to_string(),
                    "success".to_string(),
                    "error".to_string(),
                ],
                transitions: vec![
                    Transition {
                        from: "idle".to_string(),
                        to: "hover".to_string(),
                        on: "hover == true".to_string(),
                        timeline: "hover".to_string(),
                    },
                    Transition {
                        from: "pressed".to_string(),
                        to: "loading".to_string(),
                        on: "pressed".to_string(),
                        timeline: "loading".to_string(),
                    },
                    Transition {
                        from: "loading".to_string(),
                        to: "success".to_string(),
                        on: "status == success".to_string(),
                        timeline: "hover".to_string(),
                    },
                ],
            }],
            bindings: vec![
                Binding {
                    name: "label".to_string(),
                    target: Uuid::from_u128(3),
                    property: "text".to_string(),
                },
                Binding {
                    name: "accent".to_string(),
                    target: Uuid::from_u128(2),
                    property: "fill".to_string(),
                },
            ],
            events: vec![
                Event {
                    name: "submit".to_string(),
                    description: "Emitted when the button is pressed.".to_string(),
                },
                Event {
                    name: "completed".to_string(),
                    description: "Emitted after success motion finishes.".to_string(),
                },
            ],
        }
    }

    pub fn sample_minimal_bot() -> Self {
        Self::generate_character(CharacterSpec::default())
    }

    pub fn empty_scene(name: &str) -> Self {
        Self {
            id: Uuid::from_u128(500),
            name: name
                .trim()
                .is_empty()
                .then_some("Untitled Strut Scene")
                .unwrap_or(name)
                .to_string(),
            artboards: vec![Artboard {
                id: Uuid::from_u128(501),
                name: "Main".to_string(),
                width: 960.0,
                height: 540.0,
                nodes: Vec::new(),
            }],
            timelines: Vec::new(),
            state_machines: Vec::new(),
            bindings: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn sample_owl_mascot() -> Self {
        Self::generate_character(CharacterSpec {
            variant: "owl-guide".to_string(),
            name: Some("Owl Mascot".to_string()),
            accent: Some("#78d64b".to_string()),
            shell: Some("#8ee15a".to_string()),
        })
    }

    pub fn generate_character(spec: CharacterSpec) -> Self {
        let variant = spec.variant.as_str();
        let accent = spec.accent.as_deref().unwrap_or(match variant {
            "owl-guide" => "#78d64b",
            "scanner-bot" => "#2dffb8",
            "celebration-bot" => "#f6d365",
            _ => "#51bfd0",
        });
        let shell = spec.shell.as_deref().unwrap_or(match variant {
            "owl-guide" => "#8ee15a",
            _ => "#f6f1e8",
        });
        let name = spec.name.unwrap_or_else(|| match variant {
            "owl-guide" => "Owl Mascot".to_string(),
            "scanner-bot" => "Scanner Bot".to_string(),
            "celebration-bot" => "Celebration Bot".to_string(),
            _ => "Minimal Bot".to_string(),
        });

        let mut nodes = if variant == "owl-guide" {
            owl_nodes(shell, accent)
        } else {
            bot_nodes(shell, accent)
        };
        if variant == "scanner-bot" {
            nodes.push(
                Node::new(190, "ScannerBadge", NodeKind::Rect)
                    .with_shape(Shape::Rect {
                        x: 598.0,
                        y: 96.0,
                        width: 42.0,
                        height: 22.0,
                        rx: 8.0,
                    })
                    .with_style(style(Some("#2dffb8"), Some("#17142f"), 4.0)),
            );
        }
        if variant == "celebration-bot" {
            nodes.push(
                Node::new(191, "Confetti", NodeKind::Group).with_children(vec![
                    Node::new(192, "ConfettiDotA", NodeKind::Ellipse)
                        .with_shape(Shape::Ellipse {
                            cx: 318.0,
                            cy: 118.0,
                            rx: 8.0,
                            ry: 8.0,
                        })
                        .with_style(style(Some("#f6d365"), Some("#17142f"), 3.0)),
                    Node::new(193, "ConfettiDotB", NodeKind::Ellipse)
                        .with_shape(Shape::Ellipse {
                            cx: 644.0,
                            cy: 172.0,
                            rx: 7.0,
                            ry: 7.0,
                        })
                        .with_style(style(Some("#ff6b35"), Some("#17142f"), 3.0)),
                ]),
            );
        }

        Self {
            id: Uuid::from_u128(100),
            name,
            artboards: vec![Artboard {
                id: Uuid::from_u128(101),
                name: if variant == "owl-guide" {
                    "OwlMascot".to_string()
                } else {
                    "MinimalBot".to_string()
                },
                width: 960.0,
                height: 540.0,
                nodes,
            }],
            timelines: vec![
                Timeline {
                    loops: false,
                    id: Uuid::from_u128(120),
                    name: "idle_float".to_string(),
                    duration_ms: 1400,
                    tracks: vec![
                        Track {
                            target: Uuid::from_u128(102),
                            property: "translation.y".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 700,
                                    value: PropertyValue::Number(-18.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 1400,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(103),
                            property: "scale.x".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 700,
                                    value: PropertyValue::Number(0.78),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 1400,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(103),
                            property: "opacity".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 700,
                                    value: PropertyValue::Number(0.55),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 1400,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(110),
                            property: "rotation".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 700,
                                    value: PropertyValue::Number(-4.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 1400,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(110),
                            property: "translation.y".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 240,
                                    value: PropertyValue::Number(-10.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 560,
                                    value: PropertyValue::Number(3.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 900,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(111),
                            property: "rotation".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 700,
                                    value: PropertyValue::Number(4.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 1400,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                    ],
                },
                Timeline {
                    loops: false,
                    id: Uuid::from_u128(121),
                    name: "wave".to_string(),
                    duration_ms: 900,
                    tracks: vec![
                        Track {
                            target: Uuid::from_u128(102),
                            property: "translation.y".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 180,
                                    value: PropertyValue::Number(-12.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 520,
                                    value: PropertyValue::Number(-6.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 900,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(102),
                            property: "scale.x".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 180,
                                    value: PropertyValue::Number(1.04),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 900,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(102),
                            property: "scale.y".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 180,
                                    value: PropertyValue::Number(0.96),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 900,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(110),
                            property: "rotation".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 240,
                                    value: PropertyValue::Number(8.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 560,
                                    value: PropertyValue::Number(-4.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 900,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(111),
                            property: "rotation".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 180,
                                    value: PropertyValue::Number(-58.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 360,
                                    value: PropertyValue::Number(24.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 560,
                                    value: PropertyValue::Number(-44.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 900,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(111),
                            property: "translation.x".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 180,
                                    value: PropertyValue::Number(18.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 360,
                                    value: PropertyValue::Number(4.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 560,
                                    value: PropertyValue::Number(14.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 900,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(111),
                            property: "translation.y".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 180,
                                    value: PropertyValue::Number(-34.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 360,
                                    value: PropertyValue::Number(-6.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 560,
                                    value: PropertyValue::Number(-26.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 900,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                    ],
                },
                Timeline {
                    loops: false,
                    id: Uuid::from_u128(122),
                    name: "blink".to_string(),
                    duration_ms: 420,
                    tracks: vec![Track {
                        target: Uuid::from_u128(106),
                        property: "scale.y".to_string(),
                        keyframes: vec![
                            Keyframe {
                                time_ms: 0,
                                value: PropertyValue::Number(1.0),
                                easing: Easing::Linear,
                            },
                            Keyframe {
                                time_ms: 160,
                                value: PropertyValue::Number(0.08),
                                easing: Easing::EaseIn,
                            },
                            Keyframe {
                                time_ms: 420,
                                value: PropertyValue::Number(1.0),
                                easing: Easing::EaseOut,
                            },
                        ],
                    }],
                },
                Timeline {
                    loops: false,
                    id: Uuid::from_u128(123),
                    name: "scan".to_string(),
                    duration_ms: 1200,
                    tracks: vec![
                        Track {
                            target: Uuid::from_u128(115),
                            property: "translation.y".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(-42.0),
                                    easing: Easing::Linear,
                                },
                                Keyframe {
                                    time_ms: 1200,
                                    value: PropertyValue::Number(48.0),
                                    easing: Easing::Linear,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(115),
                            property: "opacity".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::Linear,
                                },
                                Keyframe {
                                    time_ms: 140,
                                    value: PropertyValue::Number(0.62),
                                    easing: Easing::Linear,
                                },
                                Keyframe {
                                    time_ms: 980,
                                    value: PropertyValue::Number(0.62),
                                    easing: Easing::Linear,
                                },
                                Keyframe {
                                    time_ms: 1200,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::Linear,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(105),
                            property: "opacity".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 600,
                                    value: PropertyValue::Number(0.7),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 1200,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                    ],
                },
                Timeline {
                    loops: false,
                    id: Uuid::from_u128(124),
                    name: "celebrate".to_string(),
                    duration_ms: 1000,
                    tracks: vec![
                        Track {
                            target: Uuid::from_u128(102),
                            property: "translation.y".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 220,
                                    value: PropertyValue::Number(-24.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 520,
                                    value: PropertyValue::Number(8.0),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 1000,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(102),
                            property: "scale.x".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 220,
                                    value: PropertyValue::Number(1.1),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 520,
                                    value: PropertyValue::Number(0.94),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 1000,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(102),
                            property: "scale.y".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 220,
                                    value: PropertyValue::Number(0.92),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 520,
                                    value: PropertyValue::Number(1.08),
                                    easing: Easing::EaseInOut,
                                },
                                Keyframe {
                                    time_ms: 1000,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(110),
                            property: "rotation".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 260,
                                    value: PropertyValue::Number(-26.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 1000,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(111),
                            property: "rotation".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 260,
                                    value: PropertyValue::Number(26.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 1000,
                                    value: PropertyValue::Number(0.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                        Track {
                            target: Uuid::from_u128(109),
                            property: "scale".to_string(),
                            keyframes: vec![
                                Keyframe {
                                    time_ms: 0,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 260,
                                    value: PropertyValue::Number(1.35),
                                    easing: Easing::EaseOut,
                                },
                                Keyframe {
                                    time_ms: 1000,
                                    value: PropertyValue::Number(1.0),
                                    easing: Easing::EaseInOut,
                                },
                            ],
                        },
                    ],
                },
            ],
            state_machines: vec![StateMachine {
                id: Uuid::from_u128(130),
                name: if variant == "owl-guide" {
                    "OwlMoods".to_string()
                } else {
                    "BotMoods".to_string()
                },
                inputs: vec![
                    Input {
                        name: "mode".to_string(),
                        kind: InputKind::Enum,
                    },
                    Input {
                        name: "wave".to_string(),
                        kind: InputKind::Trigger,
                    },
                    Input {
                        name: "scan".to_string(),
                        kind: InputKind::Boolean,
                    },
                ],
                states: vec![
                    "idle".to_string(),
                    "float".to_string(),
                    "wave".to_string(),
                    "blink".to_string(),
                    "scan".to_string(),
                    "celebrate".to_string(),
                    "sleep".to_string(),
                ],
                transitions: vec![
                    Transition {
                        from: "idle".to_string(),
                        to: "float".to_string(),
                        on: "mode == float".to_string(),
                        timeline: "idle_float".to_string(),
                    },
                    Transition {
                        from: "idle".to_string(),
                        to: "wave".to_string(),
                        on: "wave".to_string(),
                        timeline: "wave".to_string(),
                    },
                    Transition {
                        from: "idle".to_string(),
                        to: "scan".to_string(),
                        on: "scan == true".to_string(),
                        timeline: "scan".to_string(),
                    },
                    Transition {
                        from: "idle".to_string(),
                        to: "celebrate".to_string(),
                        on: "mode == celebrate".to_string(),
                        timeline: "celebrate".to_string(),
                    },
                    Transition {
                        from: "idle".to_string(),
                        to: "blink".to_string(),
                        on: "mode == blink".to_string(),
                        timeline: "blink".to_string(),
                    },
                ],
            }],
            bindings: vec![
                Binding {
                    name: "face_glow".to_string(),
                    target: Uuid::from_u128(105),
                    property: "stroke".to_string(),
                },
                Binding {
                    name: "body_tint".to_string(),
                    target: Uuid::from_u128(108),
                    property: "fill".to_string(),
                },
            ],
            events: vec![
                Event {
                    name: "wave_started".to_string(),
                    description: "Emitted when the bot starts waving.".to_string(),
                },
                Event {
                    name: "celebration_complete".to_string(),
                    description: "Emitted after the celebration loop completes.".to_string(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    fn contains_node(nodes: &[super::Node], name: &str) -> bool {
        nodes
            .iter()
            .any(|node| node.name == name || contains_node(&node.children, name))
    }

    #[test]
    fn sample_document_has_the_mvp_state_machine() {
        let document = Document::sample_login_button();

        assert_eq!(document.artboards.len(), 1);
        assert_eq!(document.state_machines[0].states.len(), 6);
        assert_eq!(document.timelines.len(), 2);
        assert!(document.state_machines[0]
            .inputs
            .iter()
            .any(|input| input.name == "status"));
    }

    #[test]
    fn bot_sample_has_animation_states() {
        let document = Document::sample_minimal_bot();

        assert_eq!(document.name, "Minimal Bot");
        assert_eq!(document.state_machines[0].states.len(), 7);
        assert!(document
            .timelines
            .iter()
            .any(|timeline| timeline.name == "wave"));
    }

    #[test]
    fn empty_scene_has_no_generated_character_layers() {
        let document = Document::empty_scene("New Project");

        assert_eq!(document.name, "New Project");
        assert!(document.artboards[0].nodes.is_empty());
        assert!(document.timelines.is_empty());
        assert!(document.state_machines.is_empty());
    }

    #[test]
    fn owl_character_spec_builds_editable_mascot() {
        let document = Document::generate_character(super::CharacterSpec {
            variant: "owl-guide".to_string(),
            name: Some("Owl Mascot".to_string()),
            accent: Some("#78d64b".to_string()),
            shell: Some("#8ee15a".to_string()),
        });

        assert_eq!(document.name, "Owl Mascot");
        assert_eq!(document.artboards[0].name, "OwlMascot");
        assert_eq!(document.state_machines[0].name, "OwlMoods");
        assert!(contains_node(&document.artboards[0].nodes, "Beak"));
    }
}
