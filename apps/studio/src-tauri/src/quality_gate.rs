use crate::AssistantResult;

#[derive(Default)]
struct QualityStats {
    parts: usize,
    timelines: usize,
    tracks: usize,
    active_tracks: usize,
    color_fills: Vec<String>,
    part_names: Vec<String>,
    state_names: Vec<String>,
    timeline_names: Vec<String>,
}

pub fn quality_repair_prompt(user_prompt: &str, result: &AssistantResult) -> Option<String> {
    let stats = collect_stats(result)?;
    let lower_prompt = user_prompt.to_ascii_lowercase();
    let distinct_colors = distinct_count(&stats.color_fills);
    let mut reasons = Vec::new();

    if stats.parts < 8 || distinct_colors < 3 {
        reasons.push(format!(
            "too few visual parts or materials: parts={}, distinct_fill_colors={}",
            stats.parts, distinct_colors
        ));
    }
    if stats.timelines < 2 || stats.tracks < 4 || stats.active_tracks < 2 {
        reasons.push(format!(
            "too little motion: timelines={}, tracks={}, active_motion_tracks={}",
            stats.timelines, stats.tracks, stats.active_tracks
        ));
    }

    if is_coin_like_prompt(&lower_prompt) {
        if stats.parts < 14 {
            reasons.push(format!("coin/medallion flip needs at least 14 semantic parts, got {}", stats.parts));
        }
        if stats.timelines < 4 {
            reasons.push(format!("coin/medallion flip needs at least 4 timelines/states, got {}", stats.timelines));
        }
        require_named_layer(&stats, &["front", "heads", "face"], "front/heads face", &mut reasons);
        require_named_layer(&stats, &["back", "tails", "reverse"], "back/tails face", &mut reasons);
        require_named_layer(&stats, &["rim", "edge", "depth", "side"], "visible rim/depth/edge", &mut reasons);
        require_named_layer(&stats, &["shadow", "ground"], "reactive ground shadow", &mut reasons);
        require_named_layer(&stats, &["glint", "highlight", "shine", "spark"], "glint/highlight polish", &mut reasons);
        require_state(&stats, "idle", &mut reasons);
        require_state(&stats, "flip", &mut reasons);
        if lower_prompt.contains("settle") {
            require_state(&stats, "settle", &mut reasons);
        }
        if lower_prompt.contains("anticipation") {
            require_state(&stats, "anticipation", &mut reasons);
        }
    }

    for keyword in ["hover", "press", "success", "error", "loading", "settle", "anticipation"] {
        if lower_prompt.contains(keyword) {
            require_state(&stats, keyword, &mut reasons);
        }
    }

    if reasons.is_empty() {
        return None;
    }

    Some(format!(
        "The previous animation passed JSON validation but failed Strut's visual/state quality gate. Regenerate it from scratch. User request: {user_prompt}\n\nProblems detected:\n- {}\n\nHard requirements for the regenerated result:\n- Obey every explicit state named by the user. If the prompt says anticipation, flip, or settle, those exact timeline/state names must exist.\n- Do not reduce a coin/medallion to two circles. Build a complete designed object in idle.\n- Coin/medallion flips must use at least 14 semantic editable parts: ground shadow, coin rig/body, outer rim, inner bezel, rim depth/side edge, front face, front emblem/detail, back face, back emblem/detail, surface highlight, moving glint, cast shadow, and polish/spark accents.\n- Coin/medallion flips must include idle, anticipation, flip, and settle timelines when requested.\n- Use 2.5D CSS-style illusion thinking: squash/stretch, opacity swaps, layered rim depth, parallax, shadow scale/opacity, and overshoot. Avoid zero-thickness cardboard rotation.\n- Use at least 4 timelines, at least 10 active motion tracks, and at least 4 distinct material colors.\n- Return only valid Strut JSON. No markdown.",
        reasons.join("\n- ")
    ))
}

fn collect_stats(result: &AssistantResult) -> Option<QualityStats> {
    let document = match result {
        AssistantResult::DocumentCreated { document, .. } => document,
        AssistantResult::DocumentUpdated { document, .. } => document,
        AssistantResult::Chat { .. } => return None,
    };
    let mut stats = QualityStats::default();
    for artboard in &document.artboards {
        for node in &artboard.nodes {
            count_node(node, &mut stats);
        }
    }
    stats.timelines = document.timelines.len();
    for timeline in &document.timelines {
        stats.timeline_names.push(timeline.name.to_ascii_lowercase());
        stats.state_names.push(timeline.name.to_ascii_lowercase());
        for track in &timeline.tracks {
            stats.tracks += 1;
            let property = track.property.as_str();
            if matches!(property, "rotation" | "rotation.x" | "rotation.y" | "translation.x" | "translation.y" | "scale" | "scale.x" | "scale.y") {
                stats.active_tracks += 1;
            }
        }
    }
    for machine in &document.state_machines {
        for state in &machine.states {
            stats.state_names.push(state.to_ascii_lowercase());
        }
    }
    Some(stats)
}

fn count_node(node: &strut_core::Node, stats: &mut QualityStats) {
    stats.part_names.push(node.name.to_ascii_lowercase());
    if !matches!(&node.shape, strut_core::Shape::None) {
        stats.parts += 1;
    }
    if let Some(fill) = node.style.fill.as_ref() {
        let trimmed = fill.trim();
        if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("none") {
            stats.color_fills.push(trimmed.to_ascii_lowercase());
        }
    }
    for child in &node.children {
        count_node(child, stats);
    }
}

fn is_coin_like_prompt(lower_prompt: &str) -> bool {
    lower_prompt.contains("coin")
        || lower_prompt.contains("medallion")
        || lower_prompt.contains("medal")
        || lower_prompt.contains("heads")
        || lower_prompt.contains("tails")
}

fn require_named_layer(stats: &QualityStats, needles: &[&str], label: &str, reasons: &mut Vec<String>) {
    if !stats.part_names.iter().any(|name| needles.iter().any(|needle| name.contains(needle))) {
        reasons.push(format!("missing semantic layer: {label}"));
    }
}

fn require_state(stats: &QualityStats, state: &str, reasons: &mut Vec<String>) {
    if !stats.state_names.iter().any(|name| name.contains(state)) {
        reasons.push(format!("missing requested state/timeline: {state}"));
    }
}

fn distinct_count(items: &[String]) -> usize {
    let mut seen: Vec<&str> = Vec::new();
    for item in items {
        if !seen.iter().any(|seen_item| *seen_item == item.as_str()) {
            seen.push(item);
        }
    }
    seen.len()
}
