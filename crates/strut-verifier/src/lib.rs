use strut_core::Document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    pub state_machines: usize,
    pub artboards: usize,
    pub export_ready: bool,
}

pub fn verify_document(document: &Document) -> VerificationReport {
    VerificationReport {
        state_machines: document.state_machines.len(),
        artboards: document.artboards.len(),
        export_ready: !document.artboards.is_empty() && !document.state_machines.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_document_is_export_ready() {
        let report = verify_document(&Document::sample_login_button());

        assert!(report.export_ready);
        assert_eq!(report.state_machines, 1);
    }
}
