use super::source_dependency_fixture::*;
use crate::{ast::*, dialect::JavaDialect};
use portable_codegen::*;
use std::sync::Arc;

#[test]
fn linked_source_inventory_preserves_exact_registration_facts_and_shared_provenance() {
    let draft = package(7, functions(42));
    let ready = certify(draft.clone());
    let item = &ready.ast().files()[0].items()[0];
    assert_eq!(item.source_inventory.iter().len(), 5);
    assert_eq!(
        item.source_inventory,
        JavaSourceInventory::derive(&draft, &item.item).unwrap()
    );
    for (id, value) in item.source_inventory.iter() {
        match (id, value) {
            (GeneratedSymbolId::Type(id), JavaSourceDeclaration::Type(value)) => {
                assert_eq!(Some(value), draft.generated_type(*id));
                assert!(matches!(
                    value.origin,
                    GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint)
                ));
            }
            (GeneratedSymbolId::Callable(id), JavaSourceDeclaration::Callable(value)) => {
                let original = draft.callable(*id).unwrap();
                assert_eq!(original, value);
                let (GeneratedOrigin::RustSource(left), GeneratedOrigin::RustSource(right)) =
                    (&original.origin, &value.origin)
                else {
                    panic!("source provenance")
                };
                assert!(Arc::ptr_eq(left, right));
                assert!(matches!(
                    item.names
                        .get(&TargetSymbolRef::Generated(GeneratedSymbolId::Callable(
                            *id
                        ))),
                    Some(JavaResolvedName::DeclaredPath(_))
                ));
            }
            _ => panic!("fixture only declares facade and ordinary functions"),
        }
    }
}

#[test]
fn source_inventory_substitution_changes_exact_postlink_equality() {
    let first = certify(package(7, functions(42)));
    let other = certify(package(8, functions(42)));
    let item = &first.ast().files()[0].items()[0];
    for inventory in [
        JavaSourceInventory::default(),
        other.ast().files()[0].items()[0].source_inventory.clone(),
    ] {
        let mut tampered = item.clone();
        tampered.source_inventory = inventory;
        assert_ne!(&tampered, item);
    }
    // These immutable packages exercise the shared exact-item rederivation.
    verify_linked_package(first.ast()).unwrap();
    verify_linked_package(other.ast()).unwrap();
}

#[test]
fn empty_runtime_item_has_no_source_inventory() {
    let draft = TargetAstBuilder::new(JavaDialect).build();
    let item = JavaFileItem::RuntimeMembers {
        helper: crate::dialect::JavaRuntimeHelper::Core,
        members: vec![],
    };
    assert_eq!(
        JavaSourceInventory::derive(&draft, &item)
            .unwrap()
            .iter()
            .len(),
        0
    );
}

#[test]
fn projection_retains_unsupported_source_categories_instead_of_erasing_them() {
    // Projection-only fixture: semantic certification is deliberately separate.
    let reference = package(7, functions(42));
    let origin = reference.callables().next().unwrap().origin.clone();
    let source = reference.callables().next().unwrap().source.clone();
    let signature = reference.callables().next().unwrap().signature.clone();
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let owner = builder.generated_type(GeneratedType {
        name: "Contract".into(),
        kind: JavaDeclarationKind::Interface,
        visibility: JavaVisibility::Public,
        origin: origin.clone(),
        source: source.clone(),
    });
    let method = builder.interface_method(GeneratedInterfaceMethod {
        owner,
        name: "invoke".into(),
        signature,
        origin: origin.clone(),
        source: source.clone(),
    });
    let value = builder.value(GeneratedValue {
        name: "constant".into(),
        ty: TargetTypeRef::Primitive(JavaPrimitive::Int),
        visibility: JavaVisibility::Public,
        origin,
        source,
    });
    let mut item = reference.files().next().unwrap().items()[0].clone();
    let JavaFileItem::Type { declared, .. } = &mut item else {
        panic!("type")
    };
    *declared = vec![
        GeneratedSymbolId::Type(owner),
        GeneratedSymbolId::InterfaceMethod(method),
        GeneratedSymbolId::Value(value),
    ];
    let draft = builder.build();
    let inventory = JavaSourceInventory::derive(&draft, &item).unwrap();
    assert_eq!(inventory.iter().len(), 3);
    assert!(matches!(
        inventory.get(GeneratedSymbolId::Type(owner)),
        Some(JavaSourceDeclaration::Type(_))
    ));
    assert!(matches!(
        inventory.get(GeneratedSymbolId::InterfaceMethod(method)),
        Some(JavaSourceDeclaration::InterfaceMethod(_))
    ));
    assert!(matches!(
        inventory.get(GeneratedSymbolId::Value(value)),
        Some(JavaSourceDeclaration::Value(_))
    ));
}
