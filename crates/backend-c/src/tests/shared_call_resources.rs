//! Certified target calls account for actual frame owners and direct edges.
use super::{CDialect, call_fixture::fixture, project_c_package, resources};
use portable_codegen::{TargetLinker, certify_resolved_package, verify_unresolved_package};

fn measure(graph: &[Vec<usize>], locals: usize) -> resources::Measurements {
    let (registry, source) = fixture(graph, locals, true);
    let package = project_c_package(registry, vec![source]).unwrap();
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    resources::measure(&linked.files()[0].items()[0]).unwrap()
}

#[test]
fn actual_linear_frames_sum_and_diamond_branches_take_the_maximum() {
    for locals in [0, 8] {
        let chain = measure(&[vec![1], vec![2], vec![]], locals);
        assert_eq!(chain.function_frames.len(), 3);
        assert_eq!(
            chain.frame_bound,
            chain.function_frames.values().sum::<u64>()
        );
        let diamond = measure(&[vec![1, 2], vec![3], vec![3], vec![]], locals);
        let frame = |index| {
            *diamond
                .function_frames
                .iter()
                .find(|(function, _)| function.key().name.as_str() == format!("call{index}"))
                .unwrap()
                .1
        };
        assert_eq!(
            diamond.frame_bound,
            frame(0) + frame(1).max(frame(2)) + frame(3)
        );
    }
}

#[test]
fn repeated_sequential_calls_count_the_callee_frame_once_per_live_path() {
    let measured = measure(&[vec![1, 1, 1], vec![]], 0);
    assert_eq!(
        measured.frame_bound,
        measured.function_frames.values().sum::<u64>()
    );
}

#[test]
fn unsupported_graphs_fail_before_a_certificate_can_escape() {
    for graph in [vec![vec![0]], vec![vec![1], vec![0]], vec![vec![], vec![]]] {
        let (registry, source) = fixture(&graph, 0, true);
        assert!(project_c_package(registry, vec![source]).is_err());
    }
}

#[test]
fn typed_calls_link_and_certify_through_the_shared_pipeline() {
    let (registry, source) = fixture(&[vec![1], vec![]], 0, true);
    let package = project_c_package(registry, vec![source]).unwrap();
    assert_eq!(
        package
            .callables()
            .filter(|callable| callable.visibility == super::CVisibility::Exported)
            .count(),
        1
    );
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let certified = certify_resolved_package(&CDialect, linked).unwrap();
    let output =
        portable_codegen::render_certified_package(&super::CStructuralRenderer, &certified)
            .unwrap();
    let portable_codegen::OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("expected C text");
    };
    assert!(text.contains("static int32_t poly_call1(int32_t);"));
    assert!(text.contains(" = poly_call1("));
}
