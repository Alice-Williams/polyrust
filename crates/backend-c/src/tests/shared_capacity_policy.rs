//! Admission errors remain distinct from unsupported syntax and render proofs.
use super::{
    CDialect,
    capacity_fixture::{self, Shape},
    project_c_package, resources,
};
use portable_codegen::{TargetLinker, verify_unresolved_package};
use portable_diagnostics::{DiagnosticCode, SourceRef};
use resources::{
    Measurements,
    policy::{self, CResourceKind as K},
};

fn measure(shape: Shape) -> Measurements {
    let (registry, source) = capacity_fixture::fixture(shape);
    measured((registry, source))
}

fn measured(
    (registry, source): (crate::ast::CFrozenRegistry, crate::ast::CSourceFile),
) -> Measurements {
    let package = project_c_package(registry, vec![source]).unwrap();
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let measured = resources::measure(&linked.files()[0].items()[0]).unwrap();
    let expected = policy::check(&measured, SourceRef::logical(["capacity"])).is_empty();
    let certificate = portable_codegen::certify_resolved_package(&CDialect, linked);
    assert_eq!(certificate.is_ok(), expected);
    if let Err(errors) = certificate {
        assert!(
            errors
                .iter()
                .all(|error| error.code == DiagnosticCode::TargetResourceLimit)
        );
    }
    measured
}

#[test]
fn many_simultaneous_locals_and_aggregate_copies_have_nonzero_frame_cost() {
    let scalar = measured(super::storage_fixture::fixture(499, 0));
    assert_eq!(scalar.automatic_objects, 500);
    assert_eq!(scalar.automatic_bytes, 2000);
    assert!(policy::check(&scalar, SourceRef::logical(["scalar"])).is_empty());
    let oversized = measured(super::storage_fixture::fixture(500, 0));
    let errors = policy::check(&oversized, SourceRef::logical(["scalar"]));
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), K::Nodes);
    assert_eq!(oversized.nodes, 4100);
    for count in [27, 28] {
        let record = measured(super::storage_fixture::fixture(count, 256));
        assert_eq!(record.automatic_objects, 1 + count as u64);
        assert_eq!(record.automatic_bytes, 4 + 1024 * count as u64);
        let errors = policy::check(&record, SourceRef::logical(["record"]));
        eprintln!("C storage probe {count}: {record:?}");
        assert_eq!(errors.is_empty(), count == 27);
        if count == 28 {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].kind(), K::NativeFrameBytes);
        }
    }
}

#[test]
fn actual_ast_boundaries_and_one_over_are_capacity_not_typing_failures() {
    for (shape, expected) in [
        (Shape::Parameters(128), K::Parameters),
        (Shape::Fields(257), K::Fields),
        (Shape::IdentifierBytes(257), K::IdentifierBytes),
        (Shape::CommentBytes(1024 * 1024 + 1), K::CommentBytes),
        (Shape::Conversions(92), K::Nesting),
    ] {
        let measured = measure(shape);
        let source = SourceRef::logical(["c", "capacity.c"]);
        let errors = policy::check(&measured, source.clone());
        let error = errors
            .iter()
            .find(|error| error.kind() == expected)
            .unwrap();
        assert_eq!(error.used(), expected.limit() + 1);
        assert_eq!(error.diagnostic().code, DiagnosticCode::TargetResourceLimit);
        assert_eq!(error.diagnostic().labels[0].source, source);
    }
    for shape in [
        Shape::Parameters(127),
        Shape::Fields(256),
        Shape::Conversions(91),
        Shape::Combined,
    ] {
        let measured = measure(shape);
        assert!(
            policy::check(&measured, SourceRef::logical(["boundary"])).is_empty(),
            "{shape:?}: {measured:?}"
        );
    }
}

#[test]
fn diagnostic_capacity_counts_normalized_presentation_not_only_input_bytes() {
    for (bytes, expected, accepted) in [
        (vec![b'a'; 4095], 4095, true),
        (vec![b'a'; 4096], 4096, false),
        (vec![0; 682], 4092, true),
        (vec![0; 683], 4098, false),
    ] {
        let measured = measured(super::platform_native_tests::diagnostic_fixture(bytes));
        assert_eq!(measured.max_diagnostic_bytes, expected);
        let errors = policy::check(&measured, SourceRef::logical(["diagnostic"]));
        assert_eq!(errors.is_empty(), accepted);
        if !accepted {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].kind(), K::DiagnosticBytes);
        }
    }
}

#[test]
fn frame_arithmetic_never_wraps_and_charges_every_storage_category() {
    let mut measured = Measurements::default();
    assert_eq!(resources::frame_bound(&measured).unwrap(), 4096);
    measured.automatic_bytes = 4;
    measured.value_bytes = 8;
    measured.automatic_objects = 1;
    measured.nodes = 10;
    assert_eq!(
        resources::frame_bound(&measured).unwrap(),
        4096 + 12 * 16 + 256 + 640
    );
    for dimension in 0..4 {
        let mut overflow = Measurements::default();
        match dimension {
            0 => overflow.automatic_bytes = u64::MAX,
            1 => overflow.value_bytes = u64::MAX,
            2 => overflow.automatic_objects = u64::MAX,
            3 => overflow.nodes = u64::MAX,
            _ => unreachable!(),
        }
        assert!(resources::frame_bound(&overflow).is_err());
    }
}

#[test]
fn aggregate_capacity_boundaries_reject_exact_one_over() {
    let source = SourceRef::logical(["capacity"]);
    for kind in [K::Nodes, K::SourceBytes, K::NativeFrameBytes] {
        let mut measured = Measurements::default();
        let slot = match kind {
            K::Nodes => &mut measured.nodes,
            K::SourceBytes => &mut measured.source_bound,
            K::NativeFrameBytes => &mut measured.frame_bound,
            _ => unreachable!(),
        };
        *slot = kind.limit();
        assert!(policy::check(&measured, source.clone()).is_empty());
        match kind {
            K::Nodes => measured.nodes += 1,
            K::SourceBytes => measured.source_bound += 1,
            K::NativeFrameBytes => measured.frame_bound += 1,
            _ => unreachable!(),
        }
        let errors = policy::check(&measured, source.clone());
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind(), kind);
    }
}
