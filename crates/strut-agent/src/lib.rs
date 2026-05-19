use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterKind {
    CloudModel,
    LocalModel,
    LocalAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentAdapter {
    pub id: String,
    pub name: String,
    pub kind: AdapterKind,
}

pub fn planned_adapters() -> Vec<AgentAdapter> {
    vec![
        AgentAdapter {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            kind: AdapterKind::LocalModel,
        },
        AgentAdapter {
            id: "openai-compatible".to_string(),
            name: "OpenAI-compatible".to_string(),
            kind: AdapterKind::CloudModel,
        },
        AgentAdapter {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            kind: AdapterKind::LocalAgent,
        },
        AgentAdapter {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            kind: AdapterKind::LocalAgent,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_is_first_class() {
        assert!(planned_adapters()
            .iter()
            .any(|adapter| adapter.id == "ollama"
                && matches!(adapter.kind, AdapterKind::LocalModel)));
    }
}
