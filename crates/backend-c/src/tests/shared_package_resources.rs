//! Actual source trees hit aggregate boundaries even when each file fits.
use super::{CDialect, package_fixture, package_projection_tests::linked, resources};
use crate::ast::{CComment, CDeclarations, CFileItem};
use portable_codegen::certify_resolved_package;
use portable_diagnostics::SourceRef;

fn comments(fixture: &mut package_fixture::Fixture, index: usize, comments: Vec<CComment>) {
    let file = &fixture.files[index];
    let mut items = file.items().to_vec();
    items.extend(comments.into_iter().map(CFileItem::Comment));
    fixture.files[index] =
        CDeclarations::new(fixture.registry.registrations(), file.identity().clone())
            .unwrap()
            .source_file(items)
            .unwrap();
}

#[test]
fn two_file_comment_budget_is_aggregate_with_exact_and_one_over_controls() {
    let limit = resources::policy::CResourceKind::CommentBytes.limit();
    for extra in [0, 1] {
        let mut fixture = package_fixture::fixture();
        comments(
            &mut fixture,
            0,
            vec![CComment::new(&"a".repeat((limit / 2) as usize))],
        );
        comments(
            &mut fixture,
            1,
            vec![CComment::new(&"b".repeat((limit / 2 + extra) as usize))],
        );
        let linked = linked(&fixture);
        let measured = resources::measure_package(&linked).unwrap();
        assert_eq!(measured.total.comment_bytes, limit + extra);
        assert!(measured.files.values().all(|file| {
            resources::policy::check(file, SourceRef::logical(["c", "individual-control"]))
                .is_empty()
        }));
        let certificate = certify_resolved_package(&CDialect, linked);
        assert_eq!(certificate.is_ok(), extra == 0);
        if extra != 0 {
            assert!(
                certificate
                    .unwrap_err()
                    .iter()
                    .any(|error| error.message.contains("CommentBytes"))
            );
        }
    }
}

#[test]
fn two_file_node_budget_does_not_reset_at_the_header_boundary() {
    let fixture = package_fixture::fixture();
    let baseline = resources::measure_package(&linked(&fixture)).unwrap();
    let limit = resources::policy::CResourceKind::Nodes.limit();
    let remaining = limit - baseline.total.nodes;
    for extra in [0, 1] {
        let mut fixture = package_fixture::fixture();
        comments(
            &mut fixture,
            0,
            vec![CComment::new(""); (remaining / 2) as usize],
        );
        comments(
            &mut fixture,
            1,
            vec![CComment::new(""); (remaining - remaining / 2 + extra) as usize],
        );
        let linked = linked(&fixture);
        let measured = resources::measure_package(&linked).unwrap();
        assert_eq!(measured.total.nodes, limit + extra);
        assert!(measured.files.values().all(|file| file.nodes < limit));
        // Header comments charge the actual function frames conservatively,
        // but never add a header frame or lose the cross-file public body.
        assert_eq!(measured.total.function_frames.len(), 2);
        assert!(measured.total.frame_bound > baseline.total.frame_bound);
        assert_eq!(
            certify_resolved_package(&CDialect, linked).is_ok(),
            extra == 0
        );
    }
}
