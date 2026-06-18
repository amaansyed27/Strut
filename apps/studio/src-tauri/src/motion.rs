use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MotionRenderer {
    SvgCss,
    DomCss,
    DomCss3d,
    SpriteCss,
    Canvas2d,
    Webgl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotionInput {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAsset {
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeComponent {
    pub id: String,
    pub name: String,
    pub html: String,
    pub css: String,
    pub js: String,
    pub states: Vec<String>,
    pub inputs: Vec<MotionInput>,
    pub assets: Vec<RuntimeAsset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    pub preview_width: u32,
    pub preview_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotionSpec {
    pub id: String,
    pub name: String,
    pub renderer: MotionRenderer,
    pub recipe: String,
    pub states: Vec<String>,
    pub inputs: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MotionArtifact {
    RuntimeComponent {
        renderer: MotionRenderer,
        spec: MotionSpec,
        component: RuntimeComponent,
        active_state: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionIntent {
    SvgDocument,
    DomCss,
    DomCss3d,
    SpriteCss,
    Canvas2d,
}

pub fn classify_motion_intent(prompt: &str) -> MotionIntent {
    let text = prompt.to_ascii_lowercase();
    if contains_any(&text, &["dice", " die", "rolling die", "coin", "card flip", "cube", "product spin", "3d button"]) {
        return MotionIntent::DomCss3d;
    }
    if contains_any(&text, &["mascot", "pet", "character", "duolingo", "sprite", "walk cycle", "jump", "wave"]) {
        return MotionIntent::SpriteCss;
    }
    if contains_any(&text, &["particle", "confetti", "liquid", "smoke", "fire", "physics"]) {
        return MotionIntent::Canvas2d;
    }
    if contains_any(&text, &["button", "toggle", "hover", "press", "success", "microinteraction"]) {
        return MotionIntent::DomCss;
    }
    MotionIntent::SvgDocument
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

pub fn motion_spec_for_prompt(prompt: &str) -> Option<MotionSpec> {
    match classify_motion_intent(prompt) {
        MotionIntent::DomCss3d => {
            let text = prompt.to_ascii_lowercase();
            let (id, name, recipe, states) = if text.contains("coin") {
                ("css3d-coin-flip", "CSS 3D Coin Flip", "dom-css3d.coin.flip", vec!["idle", "flip_heads", "flip_tails"])
            } else if text.contains("card") {
                ("css3d-card-flip", "CSS 3D Card Flip", "dom-css3d.card.flip", vec!["idle", "flip_front", "flip_back"])
            } else {
                ("css3d-die-roll", "CSS 3D Die Roll", "dom-css3d.die.roll", vec!["idle", "roll", "face_1", "face_2", "face_3", "face_4", "face_5", "face_6"])
            };
            Some(MotionSpec {
                id: id.to_string(),
                name: name.to_string(),
                renderer: MotionRenderer::DomCss3d,
                recipe: recipe.to_string(),
                states: states.into_iter().map(str::to_string).collect(),
                inputs: serde_json::json!({ "durationMs": 1500, "preview": "iframe_srcdoc" }),
            })
        }
        MotionIntent::DomCss => Some(MotionSpec {
            id: "dom-css-microinteraction".to_string(),
            name: "DOM CSS Microinteraction".to_string(),
            renderer: MotionRenderer::DomCss,
            recipe: "dom-css.microinteraction".to_string(),
            states: vec!["idle".to_string(), "hover".to_string(), "press".to_string(), "success".to_string()],
            inputs: serde_json::json!({ "durationMs": 420 }),
        }),
        MotionIntent::SpriteCss => Some(MotionSpec {
            id: "sprite-css-character".to_string(),
            name: "Sprite CSS Character".to_string(),
            renderer: MotionRenderer::SpriteCss,
            recipe: "sprite-css.character.motion".to_string(),
            states: vec!["idle".to_string(), "wave".to_string(), "jump".to_string(), "react".to_string()],
            inputs: serde_json::json!({ "atlas": true }),
        }),
        MotionIntent::Canvas2d => Some(MotionSpec {
            id: "canvas2d-effect".to_string(),
            name: "Canvas 2D Effect".to_string(),
            renderer: MotionRenderer::Canvas2d,
            recipe: "canvas2d.effect".to_string(),
            states: vec!["idle".to_string(), "burst".to_string(), "settle".to_string()],
            inputs: serde_json::json!({ "particleBudget": 160 }),
        }),
        MotionIntent::SvgDocument => None,
    }
}

#[tauri::command]
pub fn motion_spec_route(prompt: String) -> Option<MotionSpec> {
    motion_spec_for_prompt(&prompt)
}

pub fn verify_runtime_component(component: &RuntimeComponent, renderer: &MotionRenderer) -> Result<(), String> {
    if component.html.trim().is_empty() || component.css.trim().is_empty() {
        return Err("runtime component must include html and css".to_string());
    }
    if matches!(renderer, MotionRenderer::DomCss3d) {
        let css = component.css.to_ascii_lowercase();
        let html = component.html.to_ascii_lowercase();
        if !css.contains("perspective") {
            return Err("dom-css3d component is missing perspective".to_string());
        }
        if !css.contains("transform-style: preserve-3d") && !css.contains("transform-style:preserve-3d") {
            return Err("dom-css3d component is missing transform-style: preserve-3d".to_string());
        }
        if !css.contains("translatez") {
            return Err("dom-css3d component is missing translateZ depth".to_string());
        }
        if html.matches("face").count() < 2 {
            return Err("dom-css3d component should include visible face elements".to_string());
        }
    }
    Ok(())
}
