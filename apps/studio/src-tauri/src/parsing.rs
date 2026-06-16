use crate::*;
use serde_json::{Value, json};
use uuid::Uuid;

fn id_to_uuid(id: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes())
}

pub fn extract_json_objects(text: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;

    for c in text.chars() {
        if escape {
            current.push(c);
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
        } else if c == '"' {
            in_string = !in_string;
        } else if !in_string {
            if c == '{' {
                if depth == 0 {
                    current.clear();
                }
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    current.push('}');
                    objects.push(current.clone());
                    continue;
                }
            }
        }

        if depth > 0 {
            current.push(c);
        }
    }

    objects
}

fn colors_too_close(a: &str, b: &str) -> bool {
    let a = normalize_color_token(a);
    let b = normalize_color_token(b);
    if a.is_empty() || b.is_empty() {
        return false;
    }
    if a == b {
        return true;
    }
    match (hex_luminance(&a), hex_luminance(&b)) {
        (Some(left), Some(right)) => (left - right).abs() < 0.16,
        _ => false,
    }
}

fn contrasting_ink_for(fill: &str) -> &'static str {
    match hex_luminance(&normalize_color_token(fill)) {
        Some(luminance) if luminance < 0.48 => "#f8fafc",
        _ => "#111827",
    }
}

fn normalize_color_token(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace(' ', "")
        .replace("black", "#000000")
        .replace("white", "#ffffff")
}

fn hex_luminance(value: &str) -> Option<f64> {
    let hex = value.strip_prefix('#')?;
    let (r, g, b) = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            (r, g, b)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b)
        }
        _ => return None,
    };
    Some((0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b)) / 255.0)
}

fn subject_allows_mascot_anatomy(classification: &str, label: &str) -> bool {
    let classification = classification.to_lowercase();
    let label = label.to_lowercase();
    [
        "mascot",
        "character",
        "avatar",
        "person",
        "human",
        "creature",
    ]
    .iter()
    .any(|word| classification.contains(word) || label.contains(word))
}

fn semantic_token(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>()
}

fn is_mascot_anatomy_name(value: &str) -> bool {
    let token = semantic_token(value).to_lowercase();
    matches!(
        token.as_str(),
        "body"
            | "head"
            | "face"
            | "eyes"
            | "eye"
            | "arms"
            | "arm"
            | "leftarm"
            | "rightarm"
            | "legs"
            | "leg"
            | "leftleg"
            | "rightleg"
            | "torso"
            | "mouth"
            | "smile"
    )
}

fn semantic_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for token in text
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
    {
        tokens.push(token.to_string());
        let mut prefix = String::new();
        let mut suffix = String::new();
        for character in token.chars() {
            if character.is_ascii_digit() {
                suffix.push(character);
            } else if suffix.is_empty() {
                prefix.push(character);
            }
        }
        if !prefix.is_empty() && !suffix.is_empty() {
            tokens.push(prefix);
            tokens.push(suffix);
        }
    }
    tokens
}

fn part_text(part: &Value) -> String {
    let id = part.get("id").and_then(Value::as_str).unwrap_or("");
    let name = part.get("name").and_then(Value::as_str).unwrap_or("");
    let role = part.get("role").and_then(Value::as_str).unwrap_or("");
    format!("{} {} {}", id, name, role).to_ascii_lowercase()
}

fn role_is_reveal_like(value: &str) -> bool {
    let tokens = semantic_tokens(value);
    tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "reveal" | "result" | "outcome" | "variant" | "state" | "pose" | "frame"
        )
    })
}

fn part_is_reveal_candidate(part: &Value) -> bool {
    let tokens = semantic_tokens(&part_text(part));
    tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "result" | "outcome" | "variant" | "state" | "pose" | "frame" | "face" | "detail" | "glyph" | "dot"
        )
    }) && !tokens.iter().any(|token| {
        matches!(token.as_str(), "body" | "base" | "plate" | "shell" | "shadow" | "background")
    })
}

fn part_is_primary_motion_candidate(part: &Value) -> bool {
    let tokens = semantic_tokens(&part_text(part));
    tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "body" | "base" | "plate" | "shell" | "mark" | "object" | "token" | "card" | "group"
        )
    })
}

fn semantic_outcome_key_for_timeline(timeline: &Value) -> Option<Vec<String>> {
    let state_str = timeline.get("state").and_then(Value::as_str).unwrap_or("");
    let name_str = timeline.get("name").and_then(Value::as_str).unwrap_or("");
    let id_str = timeline.get("id").and_then(Value::as_str).unwrap_or("");
    let text = format!("{} {} {}", state_str, name_str, id_str);
    let tokens = semantic_tokens(&text)
        .into_iter()
        .filter(|token| !semantic_timeline_stopword(token))
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens)
    }
}

fn semantic_part_matches_outcome(part_id: &str, part_name: &str, outcome: &[String]) -> bool {
    let part_text = format!("{} {}", part_id, part_name).to_ascii_lowercase();
    let part_tokens = semantic_tokens(&part_text);
    outcome
        .iter()
        .any(|token| part_tokens.iter().any(|part_token| part_token == token))
}

fn semantic_timeline_stopword(token: &str) -> bool {
    matches!(
        token,
        "to"
            | "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "of"
            | "for"
            | "timeline"
            | "animation"
            | "motion"
            | "state"
            | "result"
            | "outcome"
            | "variant"
            | "roll"
            | "rolling"
            | "settle"
            | "settled"
            | "idle"
            | "face"
    )
}

fn semantic_variation(text: &str) -> f64 {
    let hash = text
        .bytes()
        .fold(0_u32, |hash, byte| hash.wrapping_mul(33).wrapping_add(u32::from(byte)));
    let value = f64::from(hash % 201) / 100.0 - 1.0;
    if value.abs() < 0.12 {
        0.32
    } else {
        value
    }
}

fn semantic_reveal_targets(plan: &Value) -> Vec<&Value> {
    let mut ids = std::collections::HashSet::new();
    if let Some(motion_roles) = plan.get("motion_roles").or_else(|| plan.get("motionRoles")).and_then(Value::as_array) {
        for role in motion_roles {
            let role_id = role.get("id").and_then(Value::as_str).unwrap_or("");
            let role_purpose = role.get("purpose").and_then(Value::as_str).unwrap_or("");
            if role_is_reveal_like(role_id) || role_is_reveal_like(role_purpose) {
                if let Some(part_refs) = role.get("part_refs").or_else(|| role.get("partRefs")).and_then(Value::as_array) {
                    for part_ref in part_refs {
                        if let Some(part_ref_str) = part_ref.as_str() {
                            ids.insert(part_ref_str.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut targets = Vec::new();
    if let Some(parts) = plan.get("parts").and_then(Value::as_array) {
        for part in parts {
            let part_id = part.get("id").and_then(Value::as_str).unwrap_or("");
            
            let mut part_motion_roles = Vec::new();
            if let Some(mr_array) = part.get("motion_roles").or_else(|| part.get("motionRoles")).and_then(Value::as_array) {
                for mr in mr_array {
                    if let Some(mr_str) = mr.as_str() {
                        part_motion_roles.push(mr_str);
                    }
                }
            }
            let role_reveal = part_motion_roles.iter().any(|role| role_is_reveal_like(role));
            if ids.contains(part_id) || role_reveal || part_is_reveal_candidate(part) {
                targets.push(part);
            }
        }
    }
    targets
}

fn semantic_motion_targets(plan: &Value) -> Vec<String> {
    let mut reveal_ids = std::collections::HashSet::new();
    let reveal_targets = semantic_reveal_targets(plan);
    for part in &reveal_targets {
        if let Some(id) = part.get("id").and_then(Value::as_str) {
            reveal_ids.insert(id);
        }
    }
    
    let mut targets = Vec::<String>::new();
    if let Some(motion_roles) = plan.get("motion_roles").or_else(|| plan.get("motionRoles")).and_then(Value::as_array) {
        for role in motion_roles {
            let role_id = role.get("id").and_then(Value::as_str).unwrap_or("");
            let role_purpose = role.get("purpose").and_then(Value::as_str).unwrap_or("");
            if role_is_reveal_like(role_id) || role_is_reveal_like(role_purpose) {
                continue;
            }
            if let Some(part_refs) = role.get("part_refs").or_else(|| role.get("partRefs")).and_then(Value::as_array) {
                for part_ref in part_refs {
                    if let Some(part_ref_str) = part_ref.as_str() {
                        if !targets.iter().any(|target| target == part_ref_str) {
                            targets.push(part_ref_str.to_string());
                        }
                    }
                }
            }
        }
    }
    
    if targets.is_empty() {
        if let Some(parts) = plan.get("parts").and_then(Value::as_array) {
            for part in parts {
                let part_id = part.get("id").and_then(Value::as_str).unwrap_or("");
                let text = part_text(part);
                let is_shadow = text.contains("shadow");
                let is_reveal = reveal_ids.contains(part_id);
                if !is_shadow && !is_reveal && part_is_primary_motion_candidate(part) {
                    targets.push(part_id.to_string());
                }
            }
        }
    }
    if targets.is_empty() {
        if let Some(parts) = plan.get("parts").and_then(Value::as_array) {
            if let Some(part) = parts.iter().find(|part| !part_text(part).contains("shadow")) {
                if let Some(part_id) = part.get("id").and_then(Value::as_str) {
                    targets.push(part_id.to_string());
                }
            }
        }
    }
    targets.into_iter().take(4).collect()
}

fn semantic_shadow_target(plan: &Value) -> Option<String> {
    plan.get("parts")
        .and_then(Value::as_array)?
        .iter()
        .find(|part| part_text(part).contains("shadow"))?
        .get("id")?
        .as_str()
        .map(String::from)
}

fn semantic_timeline_needs_repair(timeline: &Value) -> bool {
    let Some(tracks) = timeline.get("tracks").and_then(Value::as_array) else {
        return true;
    };
    if tracks.is_empty() {
        return true;
    }
    tracks.iter().all(|track| {
        let Some(keyframes) = track.get("keyframes").and_then(Value::as_array) else {
            return true;
        };
        if keyframes.is_empty() {
            return true;
        }
        let Some(first_val) = keyframes.first().and_then(|kf| kf.get("value").or(kf.get("val")).or(kf.get("v"))).and_then(Value::as_f64) else {
            return true;
        };
        keyframes.iter().all(|kf| {
            let val = kf.get("value").or(kf.get("val")).or(kf.get("v")).and_then(Value::as_f64).unwrap_or(0.0);
            (val - first_val).abs() < 1e-5
        })
    })
}

fn semantic_timeline_tracks(plan: &Value, timeline: &Value) -> Vec<Value> {
    let duration = timeline.get("duration_ms")
        .or(timeline.get("durationMs"))
        .or(timeline.get("duration"))
        .and_then(Value::as_f64)
        .unwrap_or(1200.0) as u32;
    let duration = duration.max(600);
    
    let outcome = semantic_outcome_key_for_timeline(timeline);
    let name_str = timeline.get("name").and_then(Value::as_str).unwrap_or("");
    let state_str = timeline.get("state").and_then(Value::as_str).unwrap_or("");
    let id_str = timeline.get("id").and_then(Value::as_str).unwrap_or("");
    let variation = semantic_variation(&format!("{} {} {}", id_str, name_str, state_str));
    let hop = -18.0 - (variation.abs() * 22.0);
    let settle = variation * 12.0;
    
    let mut tracks = Vec::new();
    
    // Get motion targets
    let motion_targets = semantic_motion_targets(plan);
    for target in motion_targets {
        // translation.y track
        tracks.push(json!({
            "target": target,
            "property": "translation.y",
            "keyframes": [
                {"time_ms": 0, "value": 0.0, "easing": "ease_out"},
                {"time_ms": duration / 3, "value": hop, "easing": "ease_out"},
                {"time_ms": (duration * 2) / 3, "value": 4.0 + variation.abs() * 6.0, "easing": "ease_in_out"},
                {"time_ms": duration, "value": 0.0, "easing": "ease_in_out"}
            ]
        }));
        // rotation track
        tracks.push(json!({
            "target": target,
            "property": "rotation",
            "keyframes": [
                {"time_ms": 0, "value": 0.0, "easing": "ease_in_out"},
                {"time_ms": (duration * 2) / 3, "value": settle * 2.0, "easing": "ease_out"},
                {"time_ms": duration, "value": settle, "easing": "ease_in_out"}
            ]
        }));
    }
    
    // Get shadow target
    if let Some(shadow) = semantic_shadow_target(plan) {
        tracks.push(json!({
            "target": shadow,
            "property": "opacity",
            "keyframes": [
                {"time_ms": 0, "value": 0.16, "easing": "ease_out"},
                {"time_ms": duration / 3, "value": 0.05, "easing": "ease_out"},
                {"time_ms": (duration * 2) / 3, "value": 0.24, "easing": "ease_in_out"},
                {"time_ms": duration, "value": 0.18, "easing": "ease_in_out"}
            ]
        }));
        tracks.push(json!({
            "target": shadow,
            "property": "scale.x",
            "keyframes": [
                {"time_ms": 0, "value": 1.0, "easing": "ease_out"},
                {"time_ms": duration / 3, "value": 0.68, "easing": "ease_out"},
                {"time_ms": duration, "value": 1.08, "easing": "ease_in_out"}
            ]
        }));
    }
    
    // Get reveal targets
    let reveal_targets = semantic_reveal_targets(plan);
    if !reveal_targets.is_empty() {
        let any_match = outcome.as_ref().map_or(false, |out_val| {
            reveal_targets.iter().any(|part| {
                let part_id = part.get("id").and_then(Value::as_str).unwrap_or("");
                let part_name = part.get("name").and_then(Value::as_str).unwrap_or("");
                semantic_part_matches_outcome(part_id, part_name, out_val)
            })
        });
        let single_reveal_target = reveal_targets.len() == 1;
        for part in &reveal_targets {
            let part_id = part.get("id").and_then(Value::as_str).unwrap_or("");
            let part_name = part.get("name").and_then(Value::as_str).unwrap_or("");
            let visible = outcome.as_ref().map_or(true, |out_val| {
                semantic_part_matches_outcome(part_id, part_name, out_val)
                    || (!any_match && single_reveal_target)
            });
            
            tracks.push(json!({
                "target": part_id,
                "property": "opacity",
                "keyframes": [
                    {"time_ms": 0, "value": if visible { 1.0 } else { 0.0 }, "easing": "linear"},
                    {"time_ms": duration, "value": if visible { 1.0 } else { 0.0 }, "easing": "linear"}
                ]
            }));
        }
    }
    
    tracks
}

pub fn document_from_generation_plan_value(value: &Value) -> Result<strut_core::Document, String> {
    let envelope = if value.get("plan").is_some() {
        value.clone()
    } else if let Some(document) = value.get("document") {
        if document.get("plan").is_some() {
            json!({
                "plan": document.get("plan").cloned().unwrap_or_else(|| json!({})),
                "operations": document.get("operations").cloned().unwrap_or_else(|| json!([]))
            })
        } else {
            value.clone()
        }
    } else if let Some(plan) = value.get("generation_plan").or_else(|| value.get("generationPlan")) {
        json!({
            "plan": plan,
            "operations": value.get("operations").cloned().unwrap_or_else(|| json!([]))
        })
    } else {
        value.clone()
    };

    let plan = envelope.get("plan").unwrap_or(&envelope);
    validate_generation_plan_floor(plan)?;
    
    // --- SEMANTIC VALIDATION ---
    let classification = plan.get("subject")
        .and_then(|s| s.get("classification"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let label = plan.get("subject")
        .and_then(|s| s.get("label"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let name = plan.get("name").and_then(Value::as_str).unwrap_or("Generated");
    
    let allows_mascot = subject_allows_mascot_anatomy(classification, label);
    
    let mut part_ids = std::collections::HashSet::new();
    if let Some(parts) = plan.get("parts").and_then(Value::as_array) {
        for part in parts {
            let id = part.get("id").and_then(Value::as_str).unwrap_or("");
            let name = part.get("name").and_then(Value::as_str).unwrap_or("");
            let _role = part.get("role").and_then(Value::as_str).unwrap_or("");

            if id.is_empty() {
                return Err("semantic parts must include non-empty id".to_string());
            }

            // Check duplicate ID
            if !part_ids.insert(id) {
                return Err(format!("duplicate part id '{}'", id));
            }

            // Check mascot anatomy
            if !allows_mascot {
                if is_mascot_anatomy_name(id) || is_mascot_anatomy_name(name) {
                    return Err("non-mascot subject cannot use mascot-only anatomy".to_string());
                }
            }

            // Validate geometry
            if let Some(geom) = part.get("geometry") {
                let kind = geom.get("kind").and_then(Value::as_str).unwrap_or("rect");
                if kind == "rect" {
                    let w = geom.get("width").and_then(Value::as_f64).unwrap_or(100.0);
                    let h = geom.get("height").and_then(Value::as_f64).unwrap_or(100.0);
                    if w <= 0.0 || h <= 0.0 {
                        return Err("invalid rect geometry: width and height must be positive".to_string());
                    }
                }
            }
        }
    }

    // Validate track targets
    if let Some(timelines) = plan.get("timelines").and_then(Value::as_array) {
        for tl in timelines {
            if let Some(tracks) = tl.get("tracks").and_then(Value::as_array) {
                for trk in tracks {
                    let target = trk.get("target").and_then(Value::as_str).unwrap_or("");
                    if !target.is_empty() && !part_ids.contains(target) {
                        return Err(format!("timeline track references missing part '{}'", target));
                    }
                }
            }
        }
    }
    // ---------------------------

    if crate::dice_repair::should_replace_with_canonical_dice(plan, classification, label, name) {
        return Ok(crate::dice_repair::canonical_dice_document(name));
    }
    
    let doc_id = Uuid::new_v4();
    let name = name.to_string();
    
    let mut flat_nodes = Vec::new();
    let mut centers = std::collections::HashMap::new();
    
    // Style safety: find base fill color
    let mut base_fill = None;
    if let Some(parts) = plan.get("parts").and_then(Value::as_array) {
        for part in parts {
            let id = part.get("id").and_then(Value::as_str).unwrap_or("unknown");
            let name = part.get("name").and_then(Value::as_str).unwrap_or(id);
            let role = part.get("role").and_then(Value::as_str).unwrap_or("");
            let text = format!("{} {} {}", id, name, role).to_ascii_lowercase();
            if text.contains("body")
                || text.contains("base")
                || text.contains("plate")
                || text.contains("shell")
                || text.contains("background")
            {
                if let Some(fill) = part.get("style").and_then(|s| s.get("fill")).and_then(Value::as_str) {
                    base_fill = Some(fill.to_string());
                    break;
                }
            }
        }
    }
    
    if let Some(parts) = plan.get("parts").and_then(Value::as_array) {
        for part in parts {
            let id_str = part.get("id").and_then(Value::as_str).unwrap_or("unknown");
            let id = id_to_uuid(id_str);
            let name = part.get("name").and_then(Value::as_str).unwrap_or(id_str).to_string();
            
            let empty_geom = json!({});
            let geom = part.get("geometry").unwrap_or(&empty_geom);
            let kind_str = geom.get("kind").and_then(Value::as_str).unwrap_or("rect");
            let node_kind = match kind_str {
                "ellipse" => strut_core::NodeKind::Ellipse,
                "path" => strut_core::NodeKind::Path,
                "text" => strut_core::NodeKind::Text,
                _ => strut_core::NodeKind::Rect,
            };
            
            // Compute shape and transform depending on geometry kind.
            // Transform.translate = center of the element in canvas space.
            // Shape coords are relative to that center (or absolute for path).
            let (shape, center_x, center_y) = match kind_str {
                "ellipse" => {
                    // cx,cy are center; rx,ry are radii
                    let cx = geom.get("cx").and_then(Value::as_f64).unwrap_or(480.0) as f32;
                    let cy = geom.get("cy").and_then(Value::as_f64).unwrap_or(270.0) as f32;
                    let rx = geom.get("rx").and_then(Value::as_f64).unwrap_or(50.0) as f32;
                    let ry = geom.get("ry").and_then(Value::as_f64).map(|v| v as f32).unwrap_or(rx);
                    // shape coords relative to transform (center at 0,0)
                    (strut_core::Shape::Ellipse { cx: 0.0, cy: 0.0, rx, ry }, cx, cy)
                }
                "path" => {
                    // Path uses absolute SVG coordinates; transform = identity (0,0)
                    let d = geom.get("d").and_then(Value::as_str).unwrap_or("").to_string();
                    (strut_core::Shape::Path { d }, 0.0, 0.0)
                }
                "text" => {
                    // x,y is the text anchor point in canvas space; use it as transform center
                    let tx = geom.get("x").and_then(Value::as_f64).unwrap_or(480.0) as f32;
                    let ty = geom.get("y").and_then(Value::as_f64).unwrap_or(270.0) as f32;
                    let value = geom.get("value").and_then(Value::as_str).unwrap_or("").to_string();
                    let size = geom.get("size").and_then(Value::as_f64).unwrap_or(24.0) as f32;
                    // shape coords at (0,0) relative to transform
                    (strut_core::Shape::Text { x: 0.0, y: 0.0, value, size }, tx, ty)
                }
                _ => {
                    // rect: x,y = top-left corner; width/height = size
                    // Center = (x + w/2, y + h/2); shape drawn at (-w/2, -h/2)
                    let x = geom.get("x").and_then(Value::as_f64).unwrap_or(430.0) as f32;
                    let y = geom.get("y").and_then(Value::as_f64).unwrap_or(220.0) as f32;
                    let w = geom.get("width").and_then(Value::as_f64).unwrap_or(100.0) as f32;
                    let h = geom.get("height").and_then(Value::as_f64).unwrap_or(100.0) as f32;
                    let rx = geom.get("rx").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                    let cx = x + w / 2.0;
                    let cy = y + h / 2.0;
                    (strut_core::Shape::Rect { x: -w / 2.0, y: -h / 2.0, width: w, height: h, rx }, cx, cy)
                }
            };
            
            let mut style = strut_core::Style::default();
            if let Some(s) = part.get("style") {
                let mut fill = s.get("fill").and_then(Value::as_str).map(|v| v.to_string());
                let mut stroke = s.get("stroke").and_then(Value::as_str).map(|v| v.to_string());
                
                if let Some(ref base) = base_fill {
                    let id = part.get("id").and_then(Value::as_str).unwrap_or("unknown");
                    let name = part.get("name").and_then(Value::as_str).unwrap_or(id);
                    let role = part.get("role").and_then(Value::as_str).unwrap_or("");
                    let text = format!("{} {} {}", id, name, role).to_ascii_lowercase();
                    
                    let is_foreground = text.contains("detail")
                        || text.contains("accent")
                        || text.contains("glyph")
                        || text.contains("text")
                        || text.contains("pip")
                        || text.contains("dot")
                        || text.contains("eye")
                        || text.contains("mark")
                        || text.contains("stroke")
                        || text.contains("line")
                        || text.contains("result")
                        || text.contains("outcome")
                        || text.contains("variant");
                        
                    if is_foreground {
                        if let Some(ref f) = fill {
                            if colors_too_close(f, base) {
                                fill = Some(contrasting_ink_for(base).to_string());
                            }
                        }
                        if let Some(ref st) = stroke {
                            if !st.eq_ignore_ascii_case("none") && colors_too_close(st, base) {
                                stroke = Some(contrasting_ink_for(base).to_string());
                            }
                        }
                    }
                }
                
                style.fill = fill;
                style.stroke = stroke;
                if let Some(sw) = s.get("stroke_width").and_then(Value::as_f64) {
                    style.stroke_width = sw as f32;
                }
                if let Some(op) = s.get("opacity").and_then(Value::as_f64) {
                    style.opacity = op as f32;
                }
            }
            
            let mut transform = strut_core::Transform::default();
            transform.translate_x = center_x;
            transform.translate_y = center_y;
            
            let node = strut_core::Node {
                id,
                name,
                kind: node_kind,
                role: part.get("role").and_then(Value::as_str).map(String::from),
                transform,
                style,
                shape,
                children: vec![],
            };
            
            let parent_id_str = part.get("parent").and_then(Value::as_str).map(String::from);
            centers.insert(id, (center_x, center_y));
            flat_nodes.push((node, parent_id_str));
        }
    }
    
    let mut flat_map = std::collections::HashMap::new();
    let mut roots = Vec::new();
    let mut child_to_parent = std::collections::HashMap::new();
    
    for (mut node, parent_id_str) in flat_nodes {
        if let Some(parent_str) = parent_id_str {
            let parent_uuid = id_to_uuid(&parent_str);
            if let Some(&(px, py)) = centers.get(&parent_uuid) {
                node.transform.translate_x -= px;
                node.transform.translate_y -= py;
            }
            child_to_parent.insert(node.id, parent_uuid);
        } else {
            roots.push(node.id);
        }
        flat_map.insert(node.id, node);
    }
    
    let mut parent_to_child_ids = std::collections::HashMap::new();
    for (&child_id, &parent_id) in &child_to_parent {
        parent_to_child_ids.entry(parent_id).or_insert_with(Vec::new).push(child_id);
    }
    
    fn build_tree(
        node_id: Uuid, 
        flat_map: &mut std::collections::HashMap<Uuid, strut_core::Node>, 
        parent_to_child_ids: &std::collections::HashMap<Uuid, Vec<Uuid>>
    ) -> strut_core::Node {
        let mut node = flat_map.remove(&node_id).unwrap();
        if let Some(child_ids) = parent_to_child_ids.get(&node_id) {
            let mut children = Vec::new();
            for &child_id in child_ids {
                children.push(build_tree(child_id, flat_map, parent_to_child_ids));
            }
            node.children = children;
        }
        node
    }
    
    let mut root_nodes = Vec::new();
    for root_id in roots {
        if flat_map.contains_key(&root_id) {
            root_nodes.push(build_tree(root_id, &mut flat_map, &parent_to_child_ids));
        }
    }
    
    let remaining_ids: Vec<Uuid> = flat_map.keys().cloned().collect();
    for id in remaining_ids {
        if flat_map.contains_key(&id) {
            root_nodes.push(build_tree(id, &mut flat_map, &parent_to_child_ids));
        }
    }
    
    let root_id = Uuid::new_v4();
    let root_node = strut_core::Node {
        id: root_id,
        name: "Root".to_string(),
        kind: strut_core::NodeKind::Group,
        role: None,
        transform: strut_core::Transform::default(),
        style: strut_core::Style::default(),
        shape: strut_core::Shape::None,
        children: root_nodes,
    };
    
    let artboards = vec![strut_core::Artboard {
        id: Uuid::new_v4(),
        name: "Main".to_string(),
        width: 960.0,
        height: 540.0,
        nodes: vec![root_node],
    }];
    
    let mut timelines = Vec::new();
    if let Some(tls) = plan.get("timelines").and_then(Value::as_array) {
        for tl in tls {
            let mut tracks = Vec::new();
            
            let needs_repair = semantic_timeline_needs_repair(tl);
            let empty_vec = vec![];
            let tracks_to_parse = if needs_repair {
                semantic_timeline_tracks(plan, tl)
            } else {
                tl.get("tracks").and_then(Value::as_array).cloned().unwrap_or(empty_vec)
            };
            
            for trk in &tracks_to_parse {
                let target_str = trk.get("target").and_then(Value::as_str).unwrap_or("");
                let target = id_to_uuid(target_str);
                let property = trk.get("property").and_then(Value::as_str).unwrap_or("").to_string();
                
                let mut keyframes = Vec::new();
                if let Some(kfs) = trk.get("keyframes").and_then(Value::as_array) {
                    for kf in kfs {
                        let time_ms = kf.get("time_ms").or(kf.get("time")).or(kf.get("t")).and_then(Value::as_f64).unwrap_or(0.0) as u32;
                        
                        // Handle both raw number values and nested {"type":"number","value":X} objects
                        let raw_val = kf.get("value").or(kf.get("val")).or(kf.get("v"));
                        let val_num = if let Some(v) = raw_val {
                            if let Some(n) = v.as_f64() {
                                n as f32  // raw number: "value": 42.0
                            } else if let Some(inner) = v.get("value").and_then(Value::as_f64) {
                                inner as f32  // nested object: "value": {"type":"number","value":42.0}
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };
                        
                        let easing_str = kf.get("easing").and_then(Value::as_str).unwrap_or("ease_in_out").to_lowercase();
                        let easing = match easing_str.as_str() {
                            "linear" => strut_core::Easing::Linear,
                            "ease_in" | "easein" => strut_core::Easing::EaseIn,
                            "ease_out" | "easeout" => strut_core::Easing::EaseOut,
                            _ => strut_core::Easing::EaseInOut,
                        };
                        
                        keyframes.push(strut_core::Keyframe {
                            time_ms,
                            value: strut_core::PropertyValue::Number(val_num),
                            easing,
                        });
                    }
                }
                
                if !keyframes.is_empty() {
                    tracks.push(strut_core::Track {
                        target,
                        property,
                        keyframes,
                    });
                }
            }
            
            timelines.push(strut_core::Timeline {
                id: Uuid::new_v4(),
                name: tl.get("name").and_then(Value::as_str).unwrap_or("Timeline").to_string(),
                duration_ms: tl.get("duration_ms")
                    .or(tl.get("durationMs"))
                    .or(tl.get("duration"))
                    .and_then(Value::as_f64)
                    .unwrap_or(1000.0) as u32,
                loops: tl.get("loops").and_then(Value::as_bool).unwrap_or(false),
                tracks,
            });
        }
    }
    
    let mut state_machines = Vec::new();
    let mut states = Vec::new();
    states.push("idle".to_string());
    if let Some(st) = plan.get("states").and_then(Value::as_array) {
        for s in st {
            let name = s.as_str().unwrap_or("");
            if name != "idle" {
                states.push(name.to_string());
            }
        }
    }
    
    state_machines.push(strut_core::StateMachine {
        id: Uuid::new_v4(),
        name: "Controller".to_string(),
        states,
        inputs: vec![],
        transitions: vec![],
    });

    Ok(strut_core::Document {
        id: doc_id,
        name,
        artboards,
        timelines,
        state_machines,
        bindings: vec![],
        events: vec![],
    })
}

fn validate_generation_plan_floor(plan: &Value) -> Result<(), String> {
    let name = plan
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if name.is_empty() {
        return Err("generation plan name is required".to_string());
    }

    let parts = plan
        .get("parts")
        .and_then(Value::as_array)
        .ok_or_else(|| "generation plan must include semantic parts".to_string())?;
    if parts.len() < 5 {
        return Err("generation plan must include at least five semantic parts".to_string());
    }

    let timelines = plan
        .get("timelines")
        .and_then(Value::as_array)
        .ok_or_else(|| "generation plan must include timelines".to_string())?;
    if timelines.is_empty() {
        return Err("generation plan must include at least one timeline".to_string());
    }

    let states = plan
        .get("states")
        .and_then(Value::as_array)
        .ok_or_else(|| "generation plan must include states".to_string())?;
    if !states
        .iter()
        .filter_map(Value::as_str)
        .any(|state| state.eq_ignore_ascii_case("idle"))
    {
        return Err("generation plan must include an idle state".to_string());
    }

    Ok(())
}

pub fn parse_provider_response_document(text: &str) -> Result<strut_core::Document, String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text.trim()) {
        if value.get("plan").is_some()
            || value.get("generation_plan").is_some()
            || value.get("generationPlan").is_some()
            || value.get("document").and_then(|document| document.get("plan")).is_some()
        {
            return document_from_generation_plan_value(&value);
        }
    }

    let json_objects = extract_json_objects(text);
    let mut last_error = None;
    
    for json_str in json_objects {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) {
            match document_from_generation_plan_value(&value) {
                Ok(doc) => return Ok(doc),
                Err(error) => last_error = Some(error),
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| {
        "Could not parse response into a valid StrutDocument or GenerationPlan.".to_string()
    }))
}

pub fn try_parse_implicit_document(text: &str) -> Option<strut_core::Document> {
    if let Ok(doc) = parse_provider_response_document(text) {
        return Some(doc);
    }
    None
}

pub fn document_from_generation_plan_text(text: &str) -> Result<strut_core::Document, String> {
    let value = serde_json::from_str::<serde_json::Value>(text).map_err(|e| e.to_string())?;
    document_from_generation_plan_value(&value)
}

fn looks_like_uuid(value: &str) -> bool {
    value.len() == 36
        && value
            .chars()
            .enumerate()
            .all(|(index, character)| match index {
                8 | 13 | 18 | 23 => character == '-',
                _ => character.is_ascii_hexdigit(),
            })
}

fn default_style_value() -> Value {
    json!({
        "fill": null,
        "stroke": null,
        "stroke_width": 0.0,
        "opacity": 1.0,
        "linecap": "round",
        "linejoin": "round"
    })
}

fn default_transform_value() -> Value {
    json!({
        "translate_x": 0.0,
        "translate_y": 0.0,
        "rotate": 0.0,
        "scale_x": 1.0,
        "scale_y": 1.0
    })
}

fn normalize_none_string(value: Option<&mut Value>) {
    if let Some(value) = value {
        if let Value::String(text) = value {
            if text.eq_ignore_ascii_case("none") || text.eq_ignore_ascii_case("transparent") {
                *value = Value::Null;
            }
        }
    }
}

fn fill_style_defaults(value: &mut Value) {
    let Value::Object(map) = value else {
        *value = json!(default_style_value());
        return;
    };
    normalize_none_string(map.get_mut("fill"));
    normalize_none_string(map.get_mut("stroke"));
    map.entry("fill").or_insert(Value::Null);
    map.entry("stroke").or_insert(Value::Null);
    map.entry("stroke_width").or_insert(json!(0.0));
    map.entry("opacity").or_insert(json!(1.0));
    map.entry("linecap").or_insert(json!("round"));
    map.entry("linejoin").or_insert(json!("round"));
}

fn fill_transform_defaults(value: &mut Value) {
    let Value::Object(map) = value else {
        *value = json!(default_transform_value());
        return;
    };
    map.entry("translate_x").or_insert(json!(0.0));
    map.entry("translate_y").or_insert(json!(0.0));
    map.entry("rotate").or_insert(json!(0.0));
    map.entry("scale_x").or_insert(json!(1.0));
    map.entry("scale_y").or_insert(json!(1.0));
}

fn normalize_state_list(value: &mut Value) {
    let Value::Array(states) = value else {
        return;
    };
    for state in states {
        if let Value::Object(map) = state {
            let name = map
                .get("name")
                .or_else(|| map.get("id"))
                .and_then(Value::as_str)
                .unwrap_or("idle")
                .to_lowercase()
                .replace(' ', "_");
            *state = Value::String(name);
        }
    }
}

fn normalize_easing(value: &mut Value) {
    let Value::String(easing) = value else {
        *value = json!("ease_in_out");
        return;
    };
    *easing = match easing.as_str() {
        "easeIn" | "easeInQuad" | "ease_in_quad" | "ease-in" => "ease_in",
        "easeOut" | "easeOutQuad" | "ease_out_quad" | "ease-out" => "ease_out",
        "easeInOut" | "easeInOutQuad" | "ease_in_out_quad" | "ease-in-out" => "ease_in_out",
        "linear" => "linear",
        "ease_in" | "ease_out" | "ease_in_out" => easing.as_str(),
        _ => "ease_in_out",
    }
    .to_string();
}

fn normalize_keyframe_value(value: &mut Value) {
    match value {
        Value::Number(_) => {
            let number = value.clone();
            *value = json!({"type": "number", "value": number});
        }
        Value::String(text) if text.starts_with('#') => {
            let color = text.clone();
            *value = json!({"type": "color", "value": color});
        }
        Value::String(text) => {
            let text_val = text.clone();
            *value = json!({"type": "text", "value": text_val});
        }
        Value::Object(map) if !map.contains_key("type") => {
            if map.contains_key("x") && map.contains_key("y") {
                let point = value.clone();
                *value = json!({"type": "point", "value": point});
            }
        }
        _ => {}
    }
}

fn normalize_track_property(value: &mut Value) {
    let Value::String(property) = value else {
        return;
    };
    *property = match property.as_str() {
        "translate_x" | "translation_x" | "x" => "translation.x",
        "translate_y" | "translation_y" | "y" => "translation.y",
        "rotate" => "rotation",
        "scale_x" => "scale.x",
        "scale_y" => "scale.y",
        other => other,
    }
    .to_string();
}

fn fill_generated_defaults(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                fill_generated_defaults(value);
            }
        }
        Value::Object(map) => {
            if map.contains_key("artboards") {
                map.entry("timelines").or_insert_with(|| json!([]));
                map.entry("state_machines").or_insert_with(|| json!([]));
                map.entry("bindings").or_insert_with(|| json!([]));
                map.entry("events").or_insert_with(|| json!([]));
            }

            if map.contains_key("tracks") {
                if let Some(duration) = map.remove("duration") {
                    map.entry("duration_ms").or_insert(duration);
                }
            }

            if map.contains_key("keyframes") {
                if let Some(property) = map.get_mut("property") {
                    normalize_track_property(property);
                }
            }

            if (map.contains_key("time") || map.contains_key("time_ms"))
                && map.contains_key("value")
            {
                if let Some(time) = map.remove("time") {
                    map.entry("time_ms").or_insert(time);
                }
                if let Some(value) = map.get_mut("value") {
                    normalize_keyframe_value(value);
                }
                if let Some(easing) = map.get_mut("easing") {
                    normalize_easing(easing);
                } else {
                    map.insert("easing".to_string(), json!("ease_in_out"));
                }
            }

            if map.contains_key("states") {
                map.entry("name").or_insert_with(|| json!("GeneratedMoods"));
                map.entry("inputs").or_insert_with(|| json!([]));
                map.entry("transitions").or_insert_with(|| json!([]));
                if let Some(states) = map.get_mut("states") {
                    normalize_state_list(states);
                }
            }

            if map.contains_key("kind") {
                map.entry("transform")
                    .or_insert_with(|| json!(default_transform_value()));
                map.entry("style")
                    .or_insert_with(|| json!(default_style_value()));
                map.entry("shape")
                    .or_insert_with(|| json!({"type": "none"}));
                map.entry("children").or_insert_with(|| json!([]));
            }

            if let Some(style) = map.get_mut("style") {
                fill_style_defaults(style);
            }
            if let Some(transform) = map.get_mut("transform") {
                fill_transform_defaults(transform);
            }

            for value in map.values_mut() {
                fill_generated_defaults(value);
            }
        }
        _ => {}
    }
}

fn normalize_generated_ids(
    value: &mut Value,
    id_map: &mut HashMap<String, String>,
    next_id: &mut u128,
) {
    match value {
        Value::Array(values) => {
            for value in values {
                normalize_generated_ids(value, id_map, next_id);
            }
        }
        Value::Object(map) => {
            for key in ["id", "target"] {
                if key == "id"
                    && (map.contains_key("timeline") || map.contains_key("timeline_id"))
                    && !map.contains_key("kind")
                {
                    continue;
                }
                if let Some(Value::String(raw_id)) = map.get_mut(key) {
                    if !looks_like_uuid(raw_id) {
                        let strict_id = id_map.entry(raw_id.clone()).or_insert_with(|| {
                            let id = format!("00000000-0000-0000-0000-{next_id:012x}");
                            *next_id += 1;
                            id
                        });
                        *raw_id = strict_id.clone();
                    }
                }
            }
            for value in map.values_mut() {
                normalize_generated_ids(value, id_map, next_id);
            }
        }
        _ => {}
    }
}

fn normalize_generated_document_value(value: &mut Value) {
    let mut id_map = HashMap::new();
    let mut next_id = 1u128;
    normalize_generated_ids(value, &mut id_map, &mut next_id);
    fill_generated_defaults(value);
}

fn is_legacy_preset_spec(value: &Value) -> bool {
    value.get("variant").is_some()
        && value.get("parts").is_none()
        && value.get("plan").is_none()
        && value.get("document").is_none()
        && value.get("generation_plan").is_none()
        && value.get("generationPlan").is_none()
}

pub fn parse_generated_document(text: &str) -> Result<strut_core::Document, String> {
    // 1. Check for legacy preset spec first
    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        if is_legacy_preset_spec(&value) {
            return Err("provider returned the old preset spec format. Strut now requires a full editable document so different prompts do not collapse into the same character.".to_string());
        }
    }
    for json_text in extract_json_objects(text) {
        if let Ok(value) = serde_json::from_str::<Value>(&json_text) {
            if is_legacy_preset_spec(&value) {
                return Err("provider returned the old preset spec format. Strut now requires a full editable document so different prompts do not collapse into the same character.".to_string());
            }
        }
    }

    // 2. Try parsing as a raw strut_core::Document (with normalization)
    if let Ok(value) = serde_json::from_str::<Value>(text.trim()) {
        if let Ok(document) = document_from_value(&value) {
            return Ok(document);
        }
    }

    let mut last_error = None;
    for json_text in extract_json_objects(text).into_iter().rev() {
        if let Ok(value) = serde_json::from_str::<Value>(&json_text) {
            match document_from_value(&value) {
                Ok(document) => return Ok(document),
                Err(error) => last_error = Some(error),
            }
        }
    }

    // 3. Try parsing as a GenerationPlan
    match parsing::parse_provider_response_document(text) {
        Ok(document) => return Ok(document),
        Err(error) => last_error = Some(error),
    }

    for json_text in extract_json_objects(text).into_iter().rev() {
        match parsing::parse_provider_response_document(&json_text) {
            Ok(document) => return Ok(document),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| "model did not return a valid Strut document".to_string()))
}

pub fn document_from_value(value: &Value) -> Result<strut_core::Document, String> {
    let mut document_value = value
        .get("document")
        .cloned()
        .unwrap_or_else(|| value.clone());
    
    normalize_generated_document_value(&mut document_value);
    
    serde_json::from_value::<strut_core::Document>(document_value)
        .map_err(|error| format!("model response was not a Strut document: {error}"))
}
