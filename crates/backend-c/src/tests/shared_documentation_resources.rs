//! Actual normalized attachments consume the existing certificate budgets.
use super::{fixture, package};
use crate::dialect::shared::{CDialect, resources};
use portable_codegen::*;
use portable_diagnostics::DiagnosticCode;

fn measured(metadata: RustSourceOrigin, expected: bool) -> resources::Measurements {
    let checked = verify_unresolved_package(&CDialect, package(metadata)).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let measured = resources::measure(&linked.files()[0].items()[0]).unwrap();
    let certificate = certify_resolved_package(&CDialect, linked);
    assert_eq!(certificate.is_ok(), expected);
    if let Ok(certificate) = certificate {
        let rendered =
            render_certified_package(&crate::dialect::shared::CStructuralRenderer, &certificate)
                .unwrap();
        let OutputContents::Text(text) = rendered.files()[0].contents() else {
            panic!("source")
        };
        assert!(text.len() as u64 <= measured.source_bound);
    } else {
        assert!(
            certificate
                .unwrap_err()
                .iter()
                .all(|error| error.code == DiagnosticCode::TargetResourceLimit)
        );
    }
    measured
}

#[test]
fn documentation_normalized_bytes_hit_exact_and_one_over_comment_limits() {
    let limit = resources::policy::CResourceKind::CommentBytes.limit() as usize;
    for (text, bytes, expected) in [
        ("x".repeat(limit), limit, true),
        ("x".repeat(limit + 1), limit + 1, false),
        ("\0".repeat(limit / 6), (limit / 6) * 6, true),
        ("\0".repeat(limit / 6 + 1), (limit / 6 + 1) * 6, false),
    ] {
        let mut metadata = fixture::bare();
        metadata.documentation.push(text);
        assert_eq!(measured(metadata, expected).comment_bytes, bytes as u64);
    }
}

#[test]
fn documentation_empty_attributes_still_consume_nodes() {
    let baseline = measured(fixture::bare(), true);
    let allowance = resources::policy::CResourceKind::Nodes.limit() - baseline.nodes;
    for (extra, expected) in [(0, true), (1, false)] {
        let mut metadata = fixture::bare();
        metadata.documentation = vec![String::new(); (allowance + extra) as usize];
        let measured = measured(metadata, expected);
        assert_eq!(measured.nodes, baseline.nodes + allowance + extra);
        assert_eq!(measured.comment_bytes, 0);
    }
}
