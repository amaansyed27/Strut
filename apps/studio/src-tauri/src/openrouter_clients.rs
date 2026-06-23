use crate::*;
use serde_json::{json, Value};

const OPENROUTER_COMPACT_PLAN_SYSTEM: &str = r#"You are Strut's compact animation planner.
Output only raw JSON. No markdown. No explanation. No full StrutDocument.
Return exactly this envelope:
{"plan":{"id":"short_id","name":"Human Name","subject":{"classification":"object|scene|ui|mascot|abstract","label":"subject"},"parts":[Part],"states":["idle","active","settle"],"timelines":[Timeline]},"operations":[]}
Part fields: id, name, role, geometry, style, motion_roles, constraints.
Geometry kinds only: rect, ellipse, path, text.
Style fields: fill, stroke, stroke_width, opacity.
Constraints must use allowed_properties: translation.x, translation.y, rotation, scale, scale.x, scale.y, opacity.
Timeline fields: id, name, duration_ms, loops, tracks.
Track fields: target, property, keyframes. Every target must match a part id.
Keyframe fields: time_ms, value as a number, easing.
Quality rules: build the exact subject from semantic editable layers; use 10-18 parts for dynamic objects; include shadow/depth/highlight/detail layers when the prompt asks for premium, reflective, 2.5D, or 3D-style motion; timelines must include active motion, not just opacity.
"#;

async fn openrouter_text(config: &ByokProviderConfig, prompt: &str, system_prompt: &str, references: &[ReferenceImageInput]) -> Result<String, String> {
    ensure_byok_config(config)?;
    let user_content = if references.is_empty() {
        json!(prompt)
    } else {
        let mut content = vec![json!({"type":"text","text": prompt_with_reference_context(prompt, references)})];
        content.extend(references.iter().map(|reference| json!({"type":"image_url","image_url":{"url":reference.data_url}})));
        json!(content)
    };
    let payload = json!({
        "model": config.model.trim(),
        "messages": [
            {"role":"system","content": system_prompt},
            {"role":"user","content": user_content}
        ],
        "temperature": 0.15,
        "stream": false,
        "max_tokens": 2048
    });
    let token = config.api_key.as_deref().unwrap_or_default();
    let json_body = post_openrouter_chat_with_curl(&config.endpoint, token, &payload)?;
    json_body.pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("OpenRouter response missing message content. Body: {}", response_preview(&json_body.to_string())))
}

fn openrouter_generation_system_prompt(context: Option<&GenerationContext>) -> String {
    let mut text = OPENROUTER_COMPACT_PLAN_SYSTEM.to_string();
    if let Some(ctx) = context {
        text.push_str("\nWorkspace context:");
        if let Some(project_name) = &ctx.project_name { text.push_str(&format!("\n- Project: {project_name}")); }
        if let Some(chat_title) = &ctx.active_chat_title { text.push_str(&format!("\n- Chat: {chat_title}")); }
        if let Some(summary) = &ctx.current_document_summary { text.push_str(&format!("\n- Current scene: {summary}")); }
    }
    text
}

fn prompt_requests_coin(prompt: &str) -> bool {
    let text = prompt.to_ascii_lowercase();
    text.contains("coin") || text.contains("coin flip") || text.contains("coin toss") || text.contains("heads") && text.contains("tails")
}

fn kf(time_ms: u32, value: f32, easing: &str) -> Value {
    json!({"time_ms": time_ms, "value": value, "easing": easing})
}

fn track(target: &str, property: &str, keyframes: Vec<Value>) -> Value {
    json!({"target": target, "property": property, "keyframes": keyframes})
}

fn tracks_for(targets: &[&str], property: &str, keyframes: Vec<Value>) -> Vec<Value> {
    targets.iter().map(|target| track(target, property, keyframes.clone())).collect()
}

fn coin_targets() -> [&'static str; 11] {
    [
        "back_depth",
        "outer_rim",
        "heads_face",
        "tails_face",
        "inner_ring",
        "ridge_left",
        "ridge_right",
        "head_mark",
        "tail_mark",
        "highlight_sweep",
        "small_glint",
    ]
}

fn premium_coin_plan(prompt: &str) -> Value {
    let coin_targets = coin_targets();
    let mut flip_tracks = Vec::<Value>::new();
    flip_tracks.extend(tracks_for(&coin_targets, "translation.y", vec![kf(0, 0.0, "ease_out"), kf(360, -116.0, "ease_out"), kf(780, -42.0, "ease_in_out"), kf(1260, 0.0, "ease_in")]));
    flip_tracks.extend(tracks_for(&coin_targets, "scale.x", vec![kf(0, 1.0, "linear"), kf(160, 0.18, "linear"), kf(315, -1.0, "linear"), kf(470, -0.18, "linear"), kf(630, 1.0, "linear"), kf(790, 0.18, "linear"), kf(945, -1.0, "linear"), kf(1100, -0.18, "linear"), kf(1260, 1.0, "linear")]));
    flip_tracks.extend(tracks_for(&coin_targets, "rotation", vec![kf(0, -7.0, "linear"), kf(315, 18.0, "linear"), kf(630, -10.0, "linear"), kf(945, 14.0, "linear"), kf(1260, 0.0, "linear")]));
    flip_tracks.push(track("ground_shadow", "scale.x", vec![kf(0, 1.0, "ease_out"), kf(360, 0.35, "ease_out"), kf(1260, 1.08, "ease_in")]));
    flip_tracks.push(track("ground_shadow", "opacity", vec![kf(0, 0.18, "ease_out"), kf(360, 0.035, "ease_out"), kf(1260, 0.2, "ease_in")]));
    flip_tracks.push(track("heads_face", "opacity", vec![kf(0, 1.0, "linear"), kf(250, 0.0, "linear"), kf(625, 1.0, "linear"), kf(900, 0.0, "linear"), kf(1260, 1.0, "linear")]));
    flip_tracks.push(track("head_mark", "opacity", vec![kf(0, 1.0, "linear"), kf(250, 0.0, "linear"), kf(625, 1.0, "linear"), kf(900, 0.0, "linear"), kf(1260, 1.0, "linear")]));
    flip_tracks.push(track("tails_face", "opacity", vec![kf(0, 0.0, "linear"), kf(250, 1.0, "linear"), kf(625, 0.0, "linear"), kf(900, 1.0, "linear"), kf(1260, 0.0, "linear")]));
    flip_tracks.push(track("tail_mark", "opacity", vec![kf(0, 0.0, "linear"), kf(250, 1.0, "linear"), kf(625, 0.0, "linear"), kf(900, 1.0, "linear"), kf(1260, 0.0, "linear")]));

    let mut heads_tracks = Vec::<Value>::new();
    heads_tracks.extend(tracks_for(&coin_targets, "translation.y", vec![kf(0, -38.0, "ease_out"), kf(360, 10.0, "ease_in_out"), kf(760, 0.0, "ease_out")]));
    heads_tracks.extend(tracks_for(&coin_targets, "scale.x", vec![kf(0, 0.22, "ease_out"), kf(280, 1.08, "ease_in_out"), kf(760, 1.0, "ease_out")]));
    heads_tracks.push(track("heads_face", "opacity", vec![kf(0, 1.0, "linear"), kf(760, 1.0, "linear")]));
    heads_tracks.push(track("head_mark", "opacity", vec![kf(0, 1.0, "linear"), kf(760, 1.0, "linear")]));
    heads_tracks.push(track("tails_face", "opacity", vec![kf(0, 0.0, "linear"), kf(760, 0.0, "linear")]));
    heads_tracks.push(track("tail_mark", "opacity", vec![kf(0, 0.0, "linear"), kf(760, 0.0, "linear")]));

    let mut tails_tracks = Vec::<Value>::new();
    tails_tracks.extend(tracks_for(&coin_targets, "translation.y", vec![kf(0, -38.0, "ease_out"), kf(360, 10.0, "ease_in_out"), kf(760, 0.0, "ease_out")]));
    tails_tracks.extend(tracks_for(&coin_targets, "scale.x", vec![kf(0, -0.22, "ease_out"), kf(280, -1.08, "ease_in_out"), kf(760, -1.0, "ease_out")]));
    tails_tracks.push(track("heads_face", "opacity", vec![kf(0, 0.0, "linear"), kf(760, 0.0, "linear")]));
    tails_tracks.push(track("head_mark", "opacity", vec![kf(0, 0.0, "linear"), kf(760, 0.0, "linear")]));
    tails_tracks.push(track("tails_face", "opacity", vec![kf(0, 1.0, "linear"), kf(760, 1.0, "linear")]));
    tails_tracks.push(track("tail_mark", "opacity", vec![kf(0, 1.0, "linear"), kf(760, 1.0, "linear")]));

    json!({
        "plan": {
            "id": "premium_coin_flip",
            "name": if prompt.trim().is_empty() { "Premium 2.5D Coin Flip" } else { "Premium 2.5D Coin Flip" },
            "subject": {"classification": "object", "label": "reflective coin flip"},
            "states": ["idle", "flip", "heads", "tails"],
            "parts": [
                {"id":"ground_shadow","name":"Ground Shadow","role":"reactive shadow","geometry":{"kind":"ellipse","cx":488,"cy":386,"rx":92,"ry":14},"style":{"fill":"#111827","stroke":null,"stroke_width":0,"opacity":0.18}},
                {"id":"back_depth","name":"Back Depth","role":"rim depth","geometry":{"kind":"ellipse","cx":489,"cy":253,"rx":72,"ry":68},"style":{"fill":"#9a6a13","stroke":"#6f4d0a","stroke_width":2,"opacity":1}},
                {"id":"outer_rim","name":"Outer Rim","role":"thick beveled rim","geometry":{"kind":"ellipse","cx":480,"cy":246,"rx":74,"ry":70},"style":{"fill":"#d4a42f","stroke":"#7a5208","stroke_width":5,"opacity":1}},
                {"id":"heads_face","name":"Heads Face","role":"front heads face","geometry":{"kind":"ellipse","cx":480,"cy":246,"rx":61,"ry":58},"style":{"fill":"#ffd65a","stroke":"#fff2a6","stroke_width":2,"opacity":1}},
                {"id":"tails_face","name":"Tails Face","role":"back tails face","geometry":{"kind":"ellipse","cx":480,"cy":246,"rx":61,"ry":58},"style":{"fill":"#f2b935","stroke":"#fff2a6","stroke_width":2,"opacity":0}},
                {"id":"inner_ring","name":"Inner Ring","role":"engraved inner ring","geometry":{"kind":"ellipse","cx":480,"cy":246,"rx":47,"ry":44},"style":{"fill":null,"stroke":"#b8860b","stroke_width":2,"opacity":0.62}},
                {"id":"ridge_left","name":"Left Edge Ridges","role":"edge ridge pattern","geometry":{"kind":"rect","x":409,"y":218,"width":6,"height":56,"rx":3},"style":{"fill":"#8b6508","stroke":null,"stroke_width":0,"opacity":0.72}},
                {"id":"ridge_right","name":"Right Edge Ridges","role":"edge ridge pattern","geometry":{"kind":"rect","x":545,"y":218,"width":6,"height":56,"rx":3},"style":{"fill":"#8b6508","stroke":null,"stroke_width":0,"opacity":0.72}},
                {"id":"head_mark","name":"H Mark","role":"heads glyph","geometry":{"kind":"text","x":463,"y":265,"value":"H","size":54},"style":{"fill":"#4b3206","stroke":null,"stroke_width":0,"opacity":1}},
                {"id":"tail_mark","name":"T Mark","role":"tails glyph","geometry":{"kind":"text","x":465,"y":265,"value":"T","size":54},"style":{"fill":"#4b3206","stroke":null,"stroke_width":0,"opacity":0}},
                {"id":"highlight_sweep","name":"Highlight Sweep","role":"reflective cool design","geometry":{"kind":"ellipse","cx":462,"cy":216,"rx":32,"ry":5},"style":{"fill":"#fff9c4","stroke":null,"stroke_width":0,"opacity":0.62}},
                {"id":"small_glint","name":"Small Glint","role":"specular glint","geometry":{"kind":"ellipse","cx":519,"cy":207,"rx":6,"ry":6},"style":{"fill":"#fffde7","stroke":null,"stroke_width":0,"opacity":0.9}}
            ],
            "timelines": [
                {"id":"coin_idle","name":"idle","state":"idle","duration_ms":1600,"loops":true,"tracks":[
                    track("highlight_sweep", "opacity", vec![kf(0, 0.34, "ease_in_out"), kf(800, 0.94, "ease_in_out"), kf(1600, 0.34, "ease_in_out")]),
                    track("ground_shadow", "scale.x", vec![kf(0, 1.0, "ease_in_out"), kf(800, 0.86, "ease_in_out"), kf(1600, 1.0, "ease_in_out")])
                ]},
                {"id":"coin_flip","name":"flip","state":"flip","duration_ms":1260,"loops":true,"tracks":flip_tracks},
                {"id":"coin_heads","name":"heads","state":"heads","duration_ms":760,"loops":false,"tracks":heads_tracks},
                {"id":"coin_tails","name":"tails","state":"tails","duration_ms":760,"loops":false,"tracks":tails_tracks}
            ]
        },
        "operations": []
    })
}

fn premium_coin_result(prompt: &str) -> Result<AssistantResult, String> {
    let document = document_from_generation_plan_value(&premium_coin_plan(prompt))?;
    Ok(AssistantResult::DocumentCreated {
        message: "Created a premium 2.5D coin flip primitive.".to_string(),
        source: "openrouter-coin-primitive".to_string(),
        document,
        plan_summary: None,
        operation_count: None,
    })
}

fn normalize_openrouter_value(mut value: Value) -> Value {
    let fallback_name = value.get("message").and_then(Value::as_str).unwrap_or("Generated animation").to_string();
    if let Some(plan) = value.get_mut("document").and_then(|document| document.get_mut("plan")) {
        if plan.get("name").and_then(Value::as_str).unwrap_or("").trim().is_empty() { plan["name"] = json!(fallback_name); }
        if plan.get("id").and_then(Value::as_str).unwrap_or("").trim().is_empty() { plan["id"] = json!("openrouter_generated_motion"); }
        if plan.get("subject").is_none() { plan["subject"] = json!({"classification":"object","label":"generated animation"}); }
    }
    value
}

fn parse_openrouter_result(text: &str) -> Result<AssistantResult, String> {
    if let Ok(document) = document_from_generation_plan_text(text) {
        return Ok(AssistantResult::DocumentCreated {
            message: "Generated compact OpenRouter animation plan.".to_string(),
            source: "openrouter-compact-plan".to_string(),
            document,
            plan_summary: None,
            operation_count: None,
        });
    }
    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        let value = normalize_openrouter_value(value);
        if let Some(kind) = value.get("kind").and_then(Value::as_str) {
            let message = value.get("message").and_then(Value::as_str).unwrap_or("").to_string();
            if kind == "chat" { return Ok(AssistantResult::Chat { message, source: "openrouter".to_string() }); }
            if matches!(kind, "document_created" | "document_updated") {
                let document_value = value.get("document").ok_or_else(|| "OpenRouter result missing document".to_string())?;
                if let Ok(document) = serde_json::from_value::<strut_core::Document>(document_value.clone()) {
                    strut_format::validate_document(&document).map_err(|error| error.to_string())?;
                    return if kind == "document_created" {
                        Ok(AssistantResult::DocumentCreated { message, source: "openrouter".to_string(), document, plan_summary: None, operation_count: None })
                    } else {
                        Ok(AssistantResult::DocumentUpdated { message, source: "openrouter".to_string(), document, plan_summary: None, operation_count: None })
                    };
                }
            }
        }
        if let Ok(result) = crate::commands::parse_assistant_result_value(value) { return Ok(result); }
    }
    crate::commands::parse_assistant_result_from_text(text)
}

#[tauri::command]
pub async fn assistant_message_openrouter_v2(prompt: String, provider: Option<GenerationProvider>, references: Option<Vec<ReferenceImageInput>>, context: Option<GenerationContext>) -> Result<AssistantResult, String> {
    let references = references.unwrap_or_default();
    let provider = provider.ok_or_else(|| "Select OpenRouter first.".to_string())?;
    let config = provider.byok.as_ref().ok_or_else(|| "OpenRouter config missing.".to_string())?;
    if should_route_to_chat_response(&prompt, context.as_ref()) {
        let chat_prompt = chat_system_prompt(&prompt, context.as_ref());
        let message = openrouter_text(config, &prompt, &chat_prompt, &references).await?;
        return Ok(AssistantResult::Chat { message, source: "openrouter".to_string() });
    }
    if references.is_empty() && prompt_requests_coin(&prompt) {
        return premium_coin_result(&prompt).map(normalize_assistant_result_layout);
    }
    let system_prompt = openrouter_generation_system_prompt(context.as_ref());
    let text = openrouter_text(config, &prompt, &system_prompt, &references).await?;
    let result = parse_openrouter_result(&text).map_err(|error| format!("OpenRouter returned invalid compact Strut plan: {error}. Response preview: {}", response_preview(&text)))?;
    Ok(normalize_assistant_result_layout(result))
}
