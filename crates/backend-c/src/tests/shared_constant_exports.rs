//! Certificate-derived foreign bindings remain separate from owned definitions.
use super::{
    CDependencyApi, CStructuralRenderer, constant_consumer_fixture, constant_export_fixture,
    constant_producer_tests::certify, owned_constant_fixture::Shape,
};
use portable_codegen::*;
use std::collections::BTreeMap;

#[test]
fn alias_only_api_retains_original_producers_without_owned_definitions() {
    let producer = constant_consumer_fixture::producer(Shape::ConstantsOnly);
    let mut values: Vec<_> = producer.constants().cloned().collect();
    values.push(values[0].clone());
    let fixture = constant_export_fixture::facade(90, &values);
    assert!(fixture.objects.is_empty() && fixture.functions.is_empty());
    assert!(fixture.registry.registrations().inventory().is_empty());
    assert_eq!(
        fixture
            .registry
            .registrations()
            .imported_constants()
            .count(),
        values.len() - 1
    );
    let api = CDependencyApi::from_certificate(certify(&fixture)).unwrap();
    assert_eq!(api.source_root().unwrap().crate_id, 90);
    assert_eq!(api.constants().count(), 0);
    assert_eq!(api.functions().count(), 0);
    let exports: BTreeMap<_, _> = api
        .foreign_constants()
        .map(|binding| {
            assert_eq!(binding.module(), api.source_root().unwrap());
            assert_eq!(binding.name().namespace, RustExportNamespace::Value);
            assert!(api.constant(binding.dependency().declaration()).is_none());
            (binding.name().name.clone(), binding.dependency().clone())
        })
        .collect();
    assert_eq!(exports.len(), values.len());
    drop(producer);
    for (index, value) in values.iter().enumerate() {
        assert_eq!(&exports[&format!("alias_{index}")], value);
        assert_ne!(
            value.package_identity().source_root().unwrap(),
            api.source_root().unwrap()
        );
    }
}

#[test]
fn facade_rendering_has_only_normal_files_includes_and_module_documentation() {
    let producer = constant_consumer_fixture::producer(Shape::ConstantsOnly);
    let values: Vec<_> = producer.constants().cloned().collect();
    let mut fixture = constant_export_fixture::facade(91, &values);
    let render = |fixture: &super::owned_constant_fixture::Fixture| {
        let certificate = certify(fixture);
        let package = render_certified_package(&CStructuralRenderer, &certificate).unwrap();
        assert_eq!(package.files().len(), 2);
        package
            .files()
            .iter()
            .map(|file| {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("C text");
                };
                (file.path().to_owned(), text.clone())
            })
            .collect::<BTreeMap<_, _>>()
    };
    let output = render(&fixture);
    fixture.files.reverse();
    assert_eq!(output, render(&fixture));
    let header = &output["polyrust_constant_facade_91.h"];
    let source = &output["polyrust_constant_facade_91.c"];
    assert_eq!(
        header.matches("#include \"polyrust_constants.h\"").count(),
        1
    );
    assert_eq!(
        source
            .matches("#include \"polyrust_constant_facade_91.h\"")
            .count(),
        1
    );
    assert!(header.contains("Public constant facade 91."));
    assert!(!source.contains("Public constant facade"));
    for text in output.values() {
        assert!(!text.contains("runtime"));
        assert!(!text.contains("poly_alias_"));
    }
}

#[test]
fn transitive_and_finite_module_aliases_keep_all_bindings_and_original_authority() {
    let producer = constant_consumer_fixture::producer(Shape::ConstantsOnly);
    let original = producer.constants().next().unwrap().clone();
    let first = constant_export_fixture::facade(92, std::slice::from_ref(&original));
    let first = CDependencyApi::from_certificate(certify(&first)).unwrap();
    let retained = first
        .foreign_constants()
        .next()
        .unwrap()
        .dependency()
        .clone();
    assert_eq!(retained, original);
    let second = constant_export_fixture::configured(93, &[retained], |_, exports| {
        constant_export_fixture::add_module_cycle(exports);
    });
    let second = CDependencyApi::from_certificate(certify(&second)).unwrap();
    assert_eq!(second.foreign_constants().count(), 2);
    for binding in second.foreign_constants() {
        assert_eq!(binding.dependency(), &original);
        assert_eq!(binding.module().crate_id, 93);
    }
}

#[test]
fn mixed_owned_and_foreign_values_from_multiple_producers_remain_disjoint() {
    let first = constant_consumer_fixture::producer(Shape::ConstantsOnly);
    let second = constant_export_fixture::mixed(100, &[], "other_producer");
    let second = CDependencyApi::from_certificate(certify(&second)).unwrap();
    let values = vec![
        first.constants().next().unwrap().clone(),
        second.constants().next().unwrap().clone(),
    ];
    let fixture = constant_export_fixture::mixed(101, &values, "own_facade_value");
    let api = CDependencyApi::from_certificate(certify(&fixture)).unwrap();
    assert_eq!(api.constants().count(), 1);
    assert_eq!(api.foreign_constants().count(), 2);
    assert_eq!(api.constants().next().unwrap().declaration().crate_id, 101);
    for export in api.foreign_constants() {
        assert!(values.contains(export.dependency()));
        assert_ne!(export.dependency().declaration().crate_id, 101);
    }
}
