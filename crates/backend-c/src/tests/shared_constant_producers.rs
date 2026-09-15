//! Certificate-derived constant views, complete public APIs and exact identities.
use super::{
    CDependencyApi, CDialect, c_defined_constants,
    owned_constant_fixture::{self, Shape},
    owned_constant_tests::linked,
};
use crate::ast::*;
use portable_codegen::{RenderReadyPackage, RustDeclarationId, certify_resolved_package};
use std::collections::BTreeSet;

pub(super) fn certify(fixture: &owned_constant_fixture::Fixture) -> RenderReadyPackage<CDialect> {
    certify_resolved_package(&CDialect, linked(fixture)).unwrap()
}

#[test]
fn constants_only_and_mixed_producers_keep_exact_values_types_and_files() {
    for shape in [Shape::ConstantsOnly, Shape::Mixed] {
        let fixture = owned_constant_fixture::fixture(shape);
        let package = certify(&fixture);
        let api = CDependencyApi::from_certificate(package.clone()).unwrap();
        assert_eq!(api.package(), &package);
        assert_eq!(api.constants().count(), fixture.objects.len());
        assert_eq!(api.functions().count(), fixture.functions.len());
        let definitions: Vec<_> = c_defined_constants(&package).collect();
        assert_eq!(definitions.len(), fixture.objects.len());
        let mut symbols = BTreeSet::new();
        for definition in definitions {
            let CGeneratedOrigin::RustSource(origin) = &definition.object().key().origin else {
                panic!("Rust source")
            };
            let value = api.constant(origin.declaration).unwrap();
            assert_eq!(value.object(), definition.object());
            assert_eq!(value.symbol(), definition.name());
            assert_eq!(value.value(), definition.value());
            assert_eq!(value.read_type(), &definition.read_type());
            assert_eq!(value.object().ty().constness(), CConstness::Const);
            assert_eq!(value.read_type().constness(), CConstness::Unqualified);
            assert_eq!(value.implementation(), definition.implementation());
            assert_eq!(value.public_header(), api.public_header());
            assert_eq!(definition.linkage(), CLinkage::External);
            assert_eq!(value.package_identity().certificate(), &package);
            assert_eq!(value.package_identity().root(), api.root());
            symbols.insert(value.symbol().clone());
        }
        symbols.extend(api.functions().map(|function| function.symbol().clone()));
        let owner = api.constants().next().unwrap().package_identity();
        assert_eq!(
            owner.public_symbols().cloned().collect::<BTreeSet<_>>(),
            symbols
        );
        assert_eq!(
            owner.stack_bound_bytes() == 0,
            shape == Shape::ConstantsOnly
        );
        assert!(
            api.constant(RustDeclarationId {
                crate_id: 999,
                definition_path_hash: 10
            })
            .is_none()
        );
        assert!(api.constant(api.root()).is_none());
        let retained = api.constants().next().unwrap().clone();
        drop(api);
        assert_eq!(retained.package_identity().certificate(), &package);
        assert_eq!(retained, retained.clone());
    }
}

#[test]
fn reversed_file_order_does_not_change_constant_views_or_symbols() {
    let mut fixture = owned_constant_fixture::fixture(Shape::Mixed);
    let before = CDependencyApi::from_certificate(certify(&fixture)).unwrap();
    fixture.files.reverse();
    let after = CDependencyApi::from_certificate(certify(&fixture)).unwrap();
    let rows = |api: &CDependencyApi| {
        api.constants()
            .map(|constant| {
                (
                    constant.declaration(),
                    constant.symbol().clone(),
                    constant.value().clone(),
                    constant.object().clone(),
                    constant.read_type().clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(rows(&before), rows(&after));
    // Equal output does not conflate separately created certificate authority.
    assert_ne!(
        before.constants().next().unwrap(),
        after.constants().next().unwrap()
    );
}

#[test]
fn same_registration_and_declaration_can_retain_distinct_literal_certificates() {
    let mut fixture = owned_constant_fixture::fixture(Shape::ConstantsOnly);
    let original = CDependencyApi::from_certificate(certify(&fixture)).unwrap();
    let object = fixture.objects[7].clone();
    let CGeneratedOrigin::RustSource(origin) = &object.key().origin else {
        panic!("Rust source")
    };
    let declaration = origin.declaration;
    let expressions = CExpressions::new(fixture.registry.registrations());
    let builder = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[1].identity().clone(),
    )
    .unwrap();
    let mut items = fixture.files[1].items().to_vec();
    items[7] = CFileItem::Definition(
        builder
            .object_definition(
                object,
                CLinkage::External,
                expressions
                    .expression_initializer(
                        expressions
                            .literal(CLiteral::Signed(CSignedLiteral::I32(17)))
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap(),
    );
    fixture.files[1] = builder.source_file(items).unwrap();
    let changed = CDependencyApi::from_certificate(certify(&fixture)).unwrap();
    let before = original.constant(declaration).unwrap();
    let after = changed.constant(declaration).unwrap();
    assert_eq!(before.object(), after.object());
    assert_eq!(before.symbol(), after.symbol());
    assert_eq!(before.value(), &CLiteral::Signed(CSignedLiteral::I32(62)));
    assert_eq!(after.value(), &CLiteral::Signed(CSignedLiteral::I32(17)));
    assert_ne!(before, after);
    assert_ne!(before.cmp(after), std::cmp::Ordering::Equal);
    assert_ne!(before.package_identity(), after.package_identity());
}

#[test]
fn synthesized_public_constants_cannot_hide_in_a_rust_function_api() {
    let fixture = owned_constant_fixture::synthesized_constants_fixture();
    let error = CDependencyApi::from_certificate(certify(&fixture)).unwrap_err();
    assert!(
        error.contains("constant lacks Rust-source provenance"),
        "{error}"
    );
}
