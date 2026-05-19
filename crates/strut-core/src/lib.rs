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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Group,
    Rect,
    Path,
    Text,
    HitArea,
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
                    Node {
                        id: Uuid::from_u128(2),
                        name: "ButtonSurface".to_string(),
                        kind: NodeKind::Rect,
                    },
                    Node {
                        id: Uuid::from_u128(3),
                        name: "Label".to_string(),
                        kind: NodeKind::Text,
                    },
                    Node {
                        id: Uuid::from_u128(4),
                        name: "SpinnerArc".to_string(),
                        kind: NodeKind::Path,
                    },
                ],
            }],
            timelines: vec![
                Timeline {
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
}

#[cfg(test)]
mod tests {
    use super::Document;

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
}
