use crate::AssistantResult;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
struct Bounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl Bounds {
    fn width(self) -> f32 { self.max_x - self.min_x }
    fn height(self) -> f32 { self.max_y - self.min_y }
    fn center_x(self) -> f32 { (self.min_x + self.max_x) / 2.0 }
    fn center_y(self) -> f32 { (self.min_y + self.max_y) / 2.0 }
    fn merge(self, other: Bounds) -> Bounds {
        Bounds { min_x: self.min_x.min(other.min_x), min_y: self.min_y.min(other.min_y), max_x: self.max_x.max(other.max_x), max_y: self.max_y.max(other.max_y) }
    }
}

pub fn normalize_assistant_result_layout(result: AssistantResult) -> AssistantResult {
    match result {
        AssistantResult::DocumentCreated { message, source, mut document, plan_summary, operation_count } => {
            normalize_document_layout(&mut document);
            AssistantResult::DocumentCreated { message, source, document, plan_summary, operation_count }
        }
        AssistantResult::DocumentUpdated { message, source, mut document, plan_summary, operation_count } => {
            normalize_document_layout(&mut document);
            AssistantResult::DocumentUpdated { message, source, document, plan_summary, operation_count }
        }
        chat => chat,
    }
}

fn normalize_document_layout(document: &mut strut_core::Document) {
    normalize_materials(document);
    ensure_action_motion(document);

    for artboard in &mut document.artboards {
        let Some(bounds) = artboard.nodes.iter().filter_map(|node| node_bounds(node, 0.0, 0.0)).reduce(|a, b| a.merge(b)) else { continue; };
        if bounds.width() <= 1.0 || bounds.height() <= 1.0 { continue; }
        let width = artboard.width.max(1.0);
        let height = artboard.height.max(1.0);
        let should_recenter = bounds.min_x < width * 0.08 || bounds.max_x > width * 0.92 || bounds.min_y < height * 0.10 || bounds.max_y > height * 0.86;
        if !should_recenter { continue; }
        let dx = (width / 2.0 - bounds.center_x()).clamp(-width * 0.45, width * 0.45);
        let dy = (height * 0.46 - bounds.center_y()).clamp(-height * 0.38, height * 0.38);
        for node in &mut artboard.nodes {
            node.transform.translate_x += dx;
            node.transform.translate_y += dy;
        }
    }
}

fn normalize_materials(document: &mut strut_core::Document) {
    for artboard in &mut document.artboards {
        for node in &mut artboard.nodes {
            normalize_node_material(node);
        }
    }
}

fn normalize_node_material(node: &mut strut_core::Node) {
    let text = format!("{} {}", node.name, node.role.as_deref().unwrap_or("")).to_ascii_lowercase();
    let fill_is_weak = fill_is_missing_or_too_dark(node.style.fill.as_deref());

    if text.contains("shadow") {
        if node.style.fill.as_deref().map_or(true, |fill| fill.eq_ignore_ascii_case("none")) {
            node.style.fill = Some("#0f172a".to_string());
        }
        node.style.opacity = node.style.opacity.min(0.30);
    } else if text.contains("gold") || text.contains("yellow") || text.contains("amber") {
        if fill_is_weak { node.style.fill = Some("#f7c948".to_string()); }
        if node.style.stroke.as_deref().map_or(true, |stroke| stroke.eq_ignore_ascii_case("none")) {
            node.style.stroke = Some("#a16207".to_string());
            node.style.stroke_width = node.style.stroke_width.max(2.0);
        }
    } else if text.contains("silver") || text.contains("chrome") || text.contains("steel") {
        if fill_is_weak { node.style.fill = Some("#cbd5e1".to_string()); }
        if node.style.stroke.as_deref().map_or(true, |stroke| stroke.eq_ignore_ascii_case("none")) {
            node.style.stroke = Some("#64748b".to_string());
            node.style.stroke_width = node.style.stroke_width.max(2.0);
        }
    } else if text.contains("rim") || text.contains("edge") || text.contains("depth") || text.contains("side") {
        if fill_is_weak { node.style.fill = Some("#b7791f".to_string()); }
        if node.style.stroke.as_deref().map_or(true, |stroke| stroke.eq_ignore_ascii_case("none")) {
            node.style.stroke = Some("#7c4a03".to_string());
            node.style.stroke_width = node.style.stroke_width.max(2.0);
        }
    } else if text.contains("highlight") || text.contains("glint") || text.contains("shine") {
        if node.style.stroke.as_deref().map_or(true, |stroke| stroke.eq_ignore_ascii_case("none")) {
            node.style.stroke = Some("#fff7cc".to_string());
            node.style.stroke_width = node.style.stroke_width.max(3.0);
        }
        node.style.opacity = node.style.opacity.max(0.55);
    } else if fill_is_weak && !matches!(&node.shape, strut_core::Shape::None) {
        node.style.fill = Some("#94a3b8".to_string());
    }

    for child in &mut node.children {
        normalize_node_material(child);
    }
}

fn fill_is_missing_or_too_dark(fill: Option<&str>) -> bool {
    let Some(fill) = fill else { return true; };
    let fill = fill.trim();
    if fill.is_empty() || fill.eq_ignore_ascii_case("none") || fill.eq_ignore_ascii_case("transparent") { return true; }
    let Some((r, g, b)) = parse_hex_rgb(fill) else { return false; };
    let luma = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    luma < 46.0
}

fn parse_hex_rgb(fill: &str) -> Option<(u8, u8, u8)> {
    let value = fill.trim().trim_start_matches('#');
    if value.len() != 6 { return None; }
    let r = u8::from_str_radix(&value[0..2], 16).ok()?;
    let g = u8::from_str_radix(&value[2..4], 16).ok()?;
    let b = u8::from_str_radix(&value[4..6], 16).ok()?;
    Some((r, g, b))
}

fn ensure_action_motion(document: &mut strut_core::Document) {
    let primary = primary_motion_target(document);
    let shadow = shadow_target(document);
    let Some(primary) = primary else { return; };
    for timeline in &mut document.timelines {
        let name = timeline.name.to_ascii_lowercase();
        if !is_action_timeline(&name) { continue; }
        let active_track_count = timeline.tracks.iter().filter(|track| track.target == primary && is_transform_track(&track.property)).count();
        if active_track_count < 2 { add_action_tracks(timeline, primary, shadow); }
    }
}

fn primary_motion_target(document: &strut_core::Document) -> Option<Uuid> {
    let mut fallback = None;
    for artboard in &document.artboards {
        for node in &artboard.nodes {
            if let Some(id) = find_primary_node(node, &mut fallback) { return Some(id); }
        }
    }
    fallback
}

fn find_primary_node(node: &strut_core::Node, fallback: &mut Option<Uuid>) -> Option<Uuid> {
    let text = format!("{} {}", node.name, node.role.as_deref().unwrap_or("")).to_ascii_lowercase();
    let is_shape = !matches!(&node.shape, strut_core::Shape::None);
    let is_shadow_or_polish = text.contains("shadow") || text.contains("glint") || text.contains("highlight") || text.contains("mark") || text.contains("text");
    if is_shape && !is_shadow_or_polish {
        if fallback.is_none() { *fallback = Some(node.id); }
        if text.contains("body") || text.contains("base") || text.contains("face") || text.contains("main") || text.contains("surface") { return Some(node.id); }
    }
    for child in &node.children {
        if let Some(id) = find_primary_node(child, fallback) { return Some(id); }
    }
    None
}

fn shadow_target(document: &strut_core::Document) -> Option<Uuid> {
    for artboard in &document.artboards {
        for node in &artboard.nodes {
            if let Some(id) = find_named_node(node, "shadow") { return Some(id); }
        }
    }
    None
}

fn find_named_node(node: &strut_core::Node, needle: &str) -> Option<Uuid> {
    let text = format!("{} {}", node.name, node.role.as_deref().unwrap_or("")).to_ascii_lowercase();
    if text.contains(needle) && !matches!(&node.shape, strut_core::Shape::None) { return Some(node.id); }
    for child in &node.children {
        if let Some(id) = find_named_node(child, needle) { return Some(id); }
    }
    None
}

fn is_action_timeline(name: &str) -> bool {
    ["flip", "spin", "roll", "bounce", "press", "reveal", "launch", "jump", "wave"].iter().any(|term| name.contains(term))
}

fn is_transform_track(property: &str) -> bool {
    matches!(property, "rotation" | "translation.x" | "translation.y" | "scale" | "scale.x" | "scale.y")
}

fn add_action_tracks(timeline: &mut strut_core::Timeline, primary: Uuid, shadow: Option<Uuid>) {
    let duration = timeline.duration_ms.max(700);
    let mid = duration / 2;
    let end = duration;
    timeline.tracks.push(number_track(primary, "translation.y", vec![(0, 0.0), (mid / 2, -42.0), (mid, -12.0), (end, 0.0)]));
    timeline.tracks.push(number_track(primary, "rotation", vec![(0, 0.0), (mid, 360.0), (end, 720.0)]));
    timeline.tracks.push(number_track(primary, "scale.x", vec![(0, 1.0), (mid / 2, 0.16), (mid, 1.08), (end, 1.0)]));
    timeline.tracks.push(number_track(primary, "scale.y", vec![(0, 1.0), (mid / 2, 1.08), (mid, 0.96), (end, 1.0)]));
    if let Some(shadow) = shadow {
        timeline.tracks.push(number_track(shadow, "opacity", vec![(0, 0.20), (mid / 2, 0.06), (end, 0.22)]));
        timeline.tracks.push(number_track(shadow, "scale.x", vec![(0, 1.0), (mid / 2, 0.64), (end, 1.08)]));
    }
}

fn number_track(target: Uuid, property: &str, frames: Vec<(u32, f32)>) -> strut_core::Track {
    strut_core::Track {
        target,
        property: property.to_string(),
        keyframes: frames.into_iter().map(|(time_ms, value)| strut_core::Keyframe { time_ms, value: strut_core::PropertyValue::Number(value), easing: strut_core::Easing::EaseInOut }).collect(),
    }
}

fn node_bounds(node: &strut_core::Node, parent_x: f32, parent_y: f32) -> Option<Bounds> {
    let tx = parent_x + node.transform.translate_x;
    let ty = parent_y + node.transform.translate_y;
    let own = shape_bounds(&node.shape, tx, ty);
    let children = node.children.iter().filter_map(|child| node_bounds(child, tx, ty));
    match (own, children.reduce(|a, b| a.merge(b))) {
        (Some(a), Some(b)) => Some(a.merge(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn shape_bounds(shape: &strut_core::Shape, tx: f32, ty: f32) -> Option<Bounds> {
    match shape {
        strut_core::Shape::Rect { x, y, width, height, .. } => Some(Bounds { min_x: tx + *x, min_y: ty + *y, max_x: tx + *x + *width, max_y: ty + *y + *height }),
        strut_core::Shape::Ellipse { cx, cy, rx, ry } => Some(Bounds { min_x: tx + *cx - *rx, min_y: ty + *cy - *ry, max_x: tx + *cx + *rx, max_y: ty + *cy + *ry }),
        strut_core::Shape::Text { x, y, value, size } => Some(Bounds { min_x: tx + *x, min_y: ty + *y - *size, max_x: tx + *x + (value.chars().count() as f32 * *size * 0.62).max(24.0), max_y: ty + *y }),
        strut_core::Shape::Sprite { frame_width, frame_height, .. } => Some(Bounds { min_x: tx, min_y: ty, max_x: tx + *frame_width, max_y: ty + *frame_height }),
        strut_core::Shape::Path { d } => path_bounds(d, tx, ty),
        strut_core::Shape::None => None,
    }
}

fn path_bounds(d: &str, tx: f32, ty: f32) -> Option<Bounds> {
    let mut nums = Vec::<f32>::new();
    let mut current = String::new();
    for ch in d.chars() {
        if ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.') { current.push(ch); }
        else if !current.is_empty() {
            if let Ok(value) = current.parse::<f32>() { nums.push(value); }
            current.clear();
        }
    }
    if !current.is_empty() {
        if let Ok(value) = current.parse::<f32>() { nums.push(value); }
    }
    if nums.len() < 2 { return None; }
    let points = nums.chunks_exact(2).collect::<Vec<_>>();
    if points.is_empty() { return None; }
    let min_x = points.iter().map(|pair| pair[0]).fold(f32::INFINITY, f32::min);
    let max_x = points.iter().map(|pair| pair[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_y = points.iter().map(|pair| pair[1]).fold(f32::INFINITY, f32::min);
    let max_y = points.iter().map(|pair| pair[1]).fold(f32::NEG_INFINITY, f32::max);
    Some(Bounds { min_x: tx + min_x, min_y: ty + min_y, max_x: tx + max_x, max_y: ty + max_y })
}
