use crate::AssistantResult;

#[derive(Default)]
struct SceneStats {
    visible_shapes: usize,
    semantic_names: usize,
    timeline_count: usize,
    active_track_count: usize,
    has_shadow: bool,
    has_depth: bool,
    has_glint: bool,
    has_face_detail: bool,
}

pub fn quality_repair_prompt(user_prompt: &str, result: &AssistantResult) -> Option<String> {
    let prompt = user_prompt.to_ascii_lowercase();
    let wants_premium = prompt.contains("premium") || prompt.contains("2.5") || prompt.contains("3d") || prompt.contains("depth");
    let wants_coin = prompt.contains("coin") || prompt.contains("medallion") || prompt.contains("token") || prompt.contains("chip");
    let wants_component = prompt.contains("button") || prompt.contains("card") || prompt.contains("component");
    if !(wants_premium || wants_coin || wants_component) {
        return None;
    }

    let Some(document) = result_document(result) else { return None; };
    let stats = scene_stats(document);
    let is_underbuilt = stats.visible_shapes < if wants_coin { 11 } else { 8 }
        || stats.semantic_names < if wants_coin { 8 } else { 6 }
        || stats.timeline_count < requested_state_count(&prompt).max(2)
        || stats.active_track_count < if wants_coin { 5 } else { 3 }
        || (wants_coin && (!stats.has_shadow || !stats.has_depth || !stats.has_glint || !stats.has_face_detail));

    if !is_underbuilt {
        return None;
    }

    let previous = serde_json::to_string(result).unwrap_or_else(|_| "<previous result unavailable>".to_string());
    let repair = format!(
        "Your previous Strut animation was visually underbuilt. It looked like a flat/simple SVG instead of a production animation.\n\nUser prompt: {user_prompt}\n\nPrevious JSON result to repair: {previous}\n\nReturn only a corrected compact JSON assistant result. Do not explain. Keep the same subject and requested states. Rebuild the document so it has a readable idle design and active motion.\n\nRequired repair rules:\n- Do not return one flat circle/blob/rectangle as the subject.\n- Use at least 11 visible semantic parts for premium circular/object prompts.\n- Add ground shadow, contact shadow, depth/side/rim layers, face detail, highlights, and glints.\n- Use layered material colors, not one pure yellow fill.\n- Add active timelines for every requested state. Use rotation.y, translation.y, scale, shadow opacity/scale, and glint opacity where relevant.\n- Preserve the current animation intent; this is a repair/update, not a new unrelated design.\n\nFor coin/medallion/token/chip prompts specifically:\nCreate named parts for Ground Shadow, Contact Shadow, Rim Depth Side, Outer Bevel, Front Face, Inner Bevel, Edge Ridges, Front Emblem, Highlight Sweep, Micro Glints, Back Face/Back Mark, and optional Motion Blur. Front and back must differ. Rim depth must be visible as side thickness or stacked edge layers, not only a stroke. Flip must rotate through front/edge/back/edge/front poses."
    );
    Some(repair)
}

fn result_document(result: &AssistantResult) -> Option<&strut_core::Document> {
    match result {
        AssistantResult::DocumentCreated { document, .. } => Some(document),
        AssistantResult::DocumentUpdated { document, .. } => Some(document),
        AssistantResult::Chat { .. } => None,
    }
}

fn scene_stats(document: &strut_core::Document) -> SceneStats {
    let mut stats = SceneStats { timeline_count: document.timelines.len(), ..SceneStats::default() };
    for artboard in &document.artboards {
        for node in &artboard.nodes {
            collect_node_stats(node, &mut stats);
        }
    }
    for timeline in &document.timelines {
        for track in &timeline.tracks {
            if track_has_motion(track) {
                stats.active_track_count += 1;
            }
        }
    }
    stats
}

fn collect_node_stats(node: &strut_core::Node, stats: &mut SceneStats) {
    let text = format!("{} {}", node.name, node.role.as_deref().unwrap_or("")).to_ascii_lowercase();
    if node.style.opacity > 0.02 && !matches!(node.shape, strut_core::Shape::None) {
        stats.visible_shapes += 1;
    }
    if text.split(|c: char| !c.is_ascii_alphanumeric()).filter(|token| token.len() > 2).count() >= 2 {
        stats.semantic_names += 1;
    }
    stats.has_shadow |= text.contains("shadow") || text.contains("cast");
    stats.has_depth |= text.contains("rim") || text.contains("edge") || text.contains("depth") || text.contains("side") || text.contains("bevel");
    stats.has_glint |= text.contains("glint") || text.contains("highlight") || text.contains("shine") || text.contains("spark");
    stats.has_face_detail |= text.contains("emblem") || text.contains("mark") || text.contains("symbol") || text.contains("front") || text.contains("back") || text.contains("face");
    for child in &node.children {
        collect_node_stats(child, stats);
    }
}

fn track_has_motion(track: &strut_core::Track) -> bool {
    let mut values = track.keyframes.iter().filter_map(|keyframe| match &keyframe.value {
        strut_core::PropertyValue::Number(value) => Some(*value),
        _ => None,
    });
    let Some(first) = values.next() else { return false; };
    values.any(|value| (value - first).abs() > 0.01)
}

fn requested_state_count(prompt: &str) -> usize {
    ["idle", "hover", "press", "loading", "success", "anticipation", "flip", "settle", "jump", "wave", "roll", "spin"]
        .iter()
        .filter(|state| prompt.contains(**state))
        .count()
}
