//! Exact layout measurements are evidence, not a native stack certificate.
use super::{CDialect, project_c_package, resources, tests::fixture};
use crate::ast::CScalarType;
use portable_codegen::{TargetLinker, verify_unresolved_package};

#[test]
fn scalar_measurements_use_the_existing_pinned_layout_model() {
    for (scalar, bytes) in [
        (CScalarType::I32, 4),
        (CScalarType::I64, 8),
        (CScalarType::Int, 4),
        (CScalarType::Bool, 1),
    ] {
        let (registry, source) = fixture(scalar);
        let package = project_c_package(registry, vec![source]).unwrap();
        let checked = verify_unresolved_package(&CDialect, package).unwrap();
        let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
        let unit = &linked.files()[0].items()[0];
        let measured = resources::measure(unit).unwrap();
        assert_eq!(
            measured.nodes,
            if scalar == CScalarType::I64 { 118 } else { 100 }
        );
        assert_eq!(measured.depth, 5);
        assert_eq!(measured.automatic_bytes, bytes);
        assert_eq!(measured.value_bytes, bytes * 2);
        assert_eq!(measured.max_parameters, 1);
        assert_eq!(measured.max_fields, 0);
        assert_eq!(measured.comment_bytes, 0);
        assert_eq!(measured, resources::measure(unit).unwrap());
    }
}

#[test]
fn record_measurements_count_pointer_slots_without_allocating_pointees() {
    let (registry, source) = super::record_fixture::fixture();
    let package = project_c_package(registry, vec![source]).unwrap();
    let checked = verify_unresolved_package(&CDialect, package).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let measured = resources::measure(&linked.files()[0].items()[0]).unwrap();
    // One i32 parameter, one one-field record, two pointer locals.
    assert_eq!(measured.automatic_bytes, 4 + 4 + 8 + 8);
    assert_eq!(measured.max_fields, 1);
    assert!(measured.comment_bytes > 0);
    // The integrated certificate also checks the conservative frame/source
    // bounds; automatic bytes by themselves are never sufficient.
    assert!(portable_codegen::certify_resolved_package(&CDialect, linked).is_ok());
}

#[test]
fn deep_reconstruction_uses_bounded_host_stack_without_disabling_checks() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let (registry, source) =
                super::capacity_fixture::fixture(super::capacity_fixture::Shape::NestedBlocks(45));
            registry
                .registrations()
                .check_local_structure(std::slice::from_ref(&source))
                .unwrap();
            let package = project_c_package(registry, vec![source]).unwrap();
            let checked = verify_unresolved_package(&CDialect, package).unwrap();
            let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
            portable_codegen::verify_linked_package(&linked).unwrap();
            assert_eq!(
                resources::measure(&linked.files()[0].items()[0])
                    .unwrap()
                    .depth,
                95
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn candidate_probes_have_the_intended_measured_dimensions() {
    use super::capacity_fixture::{Shape, fixture};
    for shape in [
        Shape::Parameters(127),
        Shape::Fields(256),
        Shape::NestedBlocks(45),
        Shape::IdentifierBytes(256),
        Shape::CommentBytes(1024 * 1024),
        Shape::Combined,
    ] {
        let (registry, source) = fixture(shape);
        let package = project_c_package(registry, vec![source]).unwrap();
        let checked = verify_unresolved_package(&CDialect, package).unwrap();
        let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
        let measured = resources::measure(&linked.files()[0].items()[0]).unwrap();
        match shape {
            Shape::Parameters(count) => assert_eq!(measured.max_parameters, count),
            Shape::Fields(count) => assert_eq!(measured.max_fields, count),
            Shape::NestedBlocks(_) => assert_eq!(measured.depth, 95),
            Shape::IdentifierBytes(bytes) => assert_eq!(measured.max_identifier_bytes, bytes),
            Shape::CommentBytes(bytes) => assert_eq!(measured.comment_bytes, bytes as u64),
            Shape::Combined => {
                assert_eq!(measured.max_parameters, 127);
                assert_eq!(measured.max_fields, 256);
                assert_eq!(measured.max_identifier_bytes, 256);
                assert_eq!(measured.comment_bytes, 1024 * 1024);
            }
            Shape::Conversions(count) => assert_eq!(measured.depth, 5 + count),
        }
        eprintln!("C resource probe {shape:?}: {measured:?}");
    }
}

#[test]
fn expression_verifier_guard_boundary_fits_the_documented_host_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let (registry, source) =
                super::capacity_fixture::fixture(super::capacity_fixture::Shape::Conversions(123));
            let package = project_c_package(registry, vec![source]).unwrap();
            let checked = verify_unresolved_package(&CDialect, package).unwrap();
            let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
            assert_eq!(
                resources::measure(&linked.files()[0].items()[0])
                    .unwrap()
                    .depth,
                128
            );
            let (registry, source) =
                super::capacity_fixture::fixture(super::capacity_fixture::Shape::Conversions(124));
            assert!(project_c_package(registry, vec![source]).is_err());
        })
        .unwrap()
        .join()
        .unwrap();
}
