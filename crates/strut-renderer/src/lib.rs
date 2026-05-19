use strut_core::Document;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    Wgpu,
    CpuFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPlan {
    pub backend: RenderBackend,
    pub artboard_count: usize,
}

pub fn plan_render(document: &Document, backend: RenderBackend) -> RenderPlan {
    RenderPlan {
        backend,
        artboard_count: document.artboards.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_plan_counts_artboards() {
        let document = Document::sample_login_button();
        let plan = plan_render(&document, RenderBackend::Wgpu);

        assert_eq!(plan.artboard_count, 1);
        assert_eq!(plan.backend, RenderBackend::Wgpu);
    }
}
