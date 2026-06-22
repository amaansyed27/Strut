use crate::AssistantResult;

#[derive(Default)]
struct QualityStats {
    parts: usize,
    timelines: usize,
    tracks: usize,
    active_tracks: usize,
    color_fills: Vec<String>,
}

pub fn quality_repair_prompt(user_prompt: &str, result: &AssistantResult) -> Option<String> {
    let stats = collect_stats(result)?;
    let distinct_colors = distinct_count(&stats.color_fills);
    let too_simple = stats.parts < 8 || distinct_colors < 3;
    let too_static = stats.timelines < 2 || stats.tracks < 4 || stats.active_tracks < 2;
    if !too_simple && !too_static {
        return None;
    }

    Some(format!(
        "The previous animation passed JSON validation but failed Strut's visual quality gate. Regenerate it from scratch. User request: {user_prompt}\n\nProblems detected: parts={}, distinct_fill_colors={}, timelines={}, tracks={}, active_motion_tracks={}.\n\nRequirements for the regenerated result: create a complete designed object in idle, use 10-22 semantic editable parts, use at least 4 visible material/detail layers, use at least 3 distinct fill colors, create 3-7 state timelines, and make the active motion visibly change rotation, perspective rotation, translation, or scale. Do not return a single circle/blob or static object. Do not hardcode any template; derive all parts and states from the user's subject.",
        stats.parts,
        distinct_colors,
        stats.timelines,
        stats.tracks,
        stats.active_tracks
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
        for track in &timeline.tracks {
            stats.tracks += 1;
            let property = track.property.as_str();
            if matches!(property, "rotation" | "rotation.x" | "rotation.y" | "translation.x" | "translation.y" | "scale" | "scale.x" | "scale.y") {
                stats.active_tracks += 1;
            }
        }
    }
    Some(stats)
}

fn count_node(node: &strut_core::Node, stats: &mut QualityStats) {
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

fn distinct_count(items: &[String]) -> usize {
    let mut seen: Vec<&str> = Vec::new();
    for item in items {
        if !seen.iter().any(|seen_item| *seen_item == item.as_str()) {
            seen.push(item);
        }
    }
    seen.len()
}
