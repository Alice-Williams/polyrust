//! Constant edges retain complete namespaces, certificate closure and call costs.
use super::{
    CDialect,
    constant_consumer_fixture::{Usage, api, fixture, producer},
    owned_constant_fixture::Shape,
    owned_constant_tests::linked,
    project_c_package, resources,
};
use portable_codegen::*;

#[test]
fn constants_only_zero_frame_evidence_composes_without_erasing_callable_costs() {
    let owner = producer(Shape::ConstantsOnly);
    let values: Vec<_> = owner.constants().cloned().collect();
    assert_eq!(values[0].package_identity().stack_bound_bytes(), 0);
    let middle = fixture(91, &values, None, Usage::Read);
    let middle_package = linked(&middle);
    let measured_middle = resources::measure_package(&middle_package).unwrap();
    assert!(measured_middle.total.frame_bound > 0);
    assert_eq!(measured_middle.total.function_frames.len(), 8);
    let middle_api = api(&middle);
    let call = middle_api.functions().next().unwrap().clone();
    assert_eq!(call.stack_bound_bytes(), measured_middle.total.frame_bound);
    let input = fixture(90, &values, Some(call), Usage::Read);
    let package = linked(&input);
    let measured = resources::measure_package(&package).unwrap();
    assert_eq!(measured.total.function_frames.len(), 9);
    assert_eq!(
        measured.total.frame_bound,
        measured.total.function_frames[&input.functions[8]] + measured_middle.total.frame_bound
    );
    assert!(certify_resolved_package(&CDialect, package).is_ok());
}

#[test]
fn shared_value_certificate_diamonds_pass_but_alternative_authorities_reject() {
    let owner = producer(Shape::ConstantsOnly);
    let other = producer(Shape::ConstantsOnly);
    let values: Vec<_> = owner.constants().cloned().collect();
    for (middle_owner, accepted) in [(&owner, true), (&other, false)] {
        let middle_values: Vec<_> = middle_owner.constants().cloned().collect();
        let middle = api(&fixture(91, &middle_values, None, Usage::Unused));
        let input = fixture(
            90,
            &values,
            Some(middle.functions().next().unwrap().clone()),
            Usage::Unused,
        );
        let result = certify_resolved_package(&CDialect, linked(&input));
        assert_eq!(result.is_ok(), accepted);
        if let Err(errors) = result {
            assert!(format!("{errors:?}").contains("different certificates for one crate"));
        }
    }
}

#[test]
fn consumer_cannot_impersonate_the_source_crate_of_an_unused_value_import() {
    let owner = producer(Shape::ConstantsOnly);
    let values: Vec<_> = owner.constants().cloned().collect();
    let input = fixture(
        owner.source_root().unwrap().crate_id,
        &values,
        None,
        Usage::Unused,
    );
    let ast = project_c_package(input.registry, input.files).unwrap();
    let checked = verify_unresolved_package(&CDialect, ast).unwrap();
    let result = TargetLinker::new(CDialect).link_ast(&checked).unwrap_err();
    assert!(format!("{result:?}").contains("consumer crate identity"));
}
