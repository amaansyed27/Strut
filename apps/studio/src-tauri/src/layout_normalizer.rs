use crate::AssistantResult;

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
        Bounds {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
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
    for artboard in &mut document.artboards {
        let Some(bounds) = artboard.nodes.iter().filter_map(|node| node_bounds(node, 0.0, 0.0)).reduce(|a, b| a.merge(b)) else {
            continue;
        };
        if bounds.width() <= 1.0 || bounds.height() <= 1.0 {
            continue;
        }

        let width = artboard.width.max(1.0);
        let height = artboard.height.max(1.0);
        let safe_min_x = width * 0.08;
        let safe_max_x = width * 0.92;
        let safe_min_y = height * 0.10;
        let safe_max_y = height * 0.86;

        let should_recenter = bounds.min_x < safe_min_x
            || bounds.max_x > safe_max_x
            || bounds.min_y < safe_min_y
            || bounds.max_y > safe_max_y;

        if !should_recenter {
            continue;
        }

        let target_x = width / 2.0;
        let target_y = height * 0.46;
        let dx = (target_x - bounds.center_x()).clamp(-width * 0.45, width * 0.45);
        let dy = (target_y - bounds.center_y()).clamp(-height * 0.38, height * 0.38);

        for node in &mut artboard.nodes {
            node.transform.translate_x += dx;
            node.transform.translate_y += dy;
        }
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
        strut_core::Shape::Rect { x, y, width, height, .. } => Some(Bounds {
            min_x: tx + *x,
            min_y: ty + *y,
            max_x: tx + *x + *width,
            max_y: ty + *y + *height,
        }),
        strut_core::Shape::Ellipse { cx, cy, rx, ry } => Some(Bounds {
            min_x: tx + *cx - *rx,
            min_y: ty + *cy - *ry,
            max_x: tx + *cx + *rx,
            max_y: ty + *cy + *ry,
        }),
        strut_core::Shape::Text { x, y, value, size } => Some(Bounds {
            min_x: tx + *x,
            min_y: ty + *y - *size,
            max_x: tx + *x + (value.chars().count() as f32 * *size * 0.62).max(24.0),
            max_y: ty + *y,
        }),
        strut_core::Shape::Path { d } => path_bounds(d, tx, ty),
        strut_core::Shape::None => None,
    }
}

fn path_bounds(d: &str, tx: f32, ty: f32) -> Option<Bounds> {
    let mut nums = Vec::<f32>::new();
    let mut current = String::new();
    for ch in d.chars() {
        if ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.') {
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(value) = current.parse::<f32>() {
                nums.push(value);
            }
            current.clear();
        }
    }
    if !current.is_empty() {
        if let Ok(value) = current.parse::<f32>() {
            nums.push(value);
        }
    }
    if nums.len() < 2 {
        return None;
    }
    let points = nums.chunks_exact(2).collect::<Vec<_>>();
    if points.is_empty() {
        return None;
    }
    let min_x = points.iter().map(|pair| pair[0]).fold(f32::INFINITY, f32::min);
    let max_x = points.iter().map(|pair| pair[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_y = points.iter().map(|pair| pair[1]).fold(f32::INFINITY, f32::min);
    let max_y = points.iter().map(|pair| pair[1]).fold(f32::NEG_INFINITY, f32::max);
    Some(Bounds { min_x: tx + min_x, min_y: ty + min_y, max_x: tx + max_x, max_y: ty + max_y })
}
