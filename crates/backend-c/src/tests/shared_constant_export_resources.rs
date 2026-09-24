//! Export-only roots retain transitive crate identity and complete symbol checks.
use super::{
    CDependencyApi, CDialect,
    constant_consumer_fixture::producer,
    constant_export_fixture::{configured_mixed, facade, mixed},
    constant_producer_tests::certify,
    owned_constant_dependency_fixture::{owner, selected},
    owned_constant_fixture::Shape,
    owned_constant_tests::linked,
    project_c_package,
};
use portable_codegen::*;

#[test]
fn zero_owned_facade_cannot_appear_in_its_original_producers_dependency_closure() {
    let dependency = owner();
    let producer = configured_mixed(71, &[], "middle_constant", |registry, _| {
        registry.import_function(selected(&dependency)).unwrap();
    });
    let producer = CDependencyApi::from_certificate(certify(&producer)).unwrap();
    let values: Vec<_> = producer.constants().cloned().collect();
    let positive = facade(72, &values);
    certify(&positive);
    let cyclic = facade(dependency.source_root().unwrap().crate_id, &values);
    assert!(cyclic.registry.registrations().inventory().is_empty());
    let error = certify_resolved_package(&CDialect, linked(&cyclic)).unwrap_err();
    assert!(
        error
            .iter()
            .any(|error| error.message.contains("consumer crate identity"))
    );
}

#[test]
fn export_only_closure_includes_unselected_transitive_public_symbols() {
    let dependency = owner();
    let first = configured_mixed(71, &[], "middle_constant", |registry, _| {
        registry.import_function(selected(&dependency)).unwrap();
    });
    let first = CDependencyApi::from_certificate(certify(&first)).unwrap();
    for (name, accepted) in [("independent_constant", true), ("poly_other_export", false)] {
        let second = CDependencyApi::from_certificate(certify(&mixed(72, &[], name))).unwrap();
        let values = [
            first.constants().next().unwrap().clone(),
            second.constants().next().unwrap().clone(),
        ];
        let facade = facade(73, &values);
        assert!(facade.registry.registrations().inventory().is_empty());
        let result = certify_resolved_package(&CDialect, linked(&facade));
        if accepted {
            result.unwrap();
        } else {
            let error = result.unwrap_err();
            assert!(
                error
                    .iter()
                    .any(|error| error.message.contains("complete public symbols collide"))
            );
        }
    }
}

#[test]
fn owned_facade_value_cannot_collide_with_unselected_direct_producer_function() {
    let producer = producer(Shape::Mixed);
    let values = [producer.constants().next().unwrap().clone()];
    let function = producer.functions().last().unwrap();
    let fixture = mixed(74, &values, function.symbol().as_str());
    let raw = project_c_package(fixture.registry, fixture.files).unwrap();
    let checked = verify_unresolved_package(&CDialect, raw).unwrap();
    let error = TargetLinker::new(CDialect).link_ast(&checked).unwrap_err();
    assert!(
        error
            .iter()
            .any(|error| error.message.contains("owned C binding collides"))
    );
}
