use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub artboards: Vec<Artboard>,
    pub state_machines: Vec<StateMachine>,
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
            }],
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
        assert!(document.state_machines[0]
            .inputs
            .iter()
            .any(|input| input.name == "status"));
    }
}
