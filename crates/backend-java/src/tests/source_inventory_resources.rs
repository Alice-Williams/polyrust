use super::*;
use crate::tests::source_dependency_fixture::*;

#[test]
fn synthesized_adapter_inventory_has_exact_and_one_over_metadata_limits() {
    use crate::ast::*;
    use crate::dialect::JavaDialect;
    use portable_codegen::*;
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let owner = builder.generated_type(GeneratedType {
        name: "Family".into(),
        kind: JavaDeclarationKind::SealedInterface,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter),
        source: crate::tests::source_record_fixture::source(),
    });
    // Projection-only metadata fixture, not a Java syntax certificate.
    let reference = package(7, functions(42));
    let mut item = reference.files().next().unwrap().items()[0].clone();
    let JavaFileItem::Type { declared, .. } = &mut item else {
        unreachable!()
    };
    *declared = vec![GeneratedSymbolId::Type(owner)];
    let inventory = JavaSourceInventory::derive(&builder.build(), &item).unwrap();
    assert_eq!(inventory.iter().len(), 1);
    check([&inventory], 1, 0, 6).unwrap();
    assert!(
        check([&inventory], 0, 0, 6)
            .unwrap_err()
            .contains("declaration")
    );
    assert!(
        check([&inventory], 1, 0, 5)
            .unwrap_err()
            .contains("name byte")
    );
}

#[test]
fn source_inventory_exact_and_one_over_limits() {
    let ready = certify(package(7, functions(42)));
    let inventory = &ready.ast().files()[0].items()[0].source_inventory;
    let bytes = "Generated".len() + 4 * "fn000000000000000a".len();
    check([inventory], 5, 6, bytes).unwrap();
    for (declarations, parameters, names, message) in [
        (4, 6, bytes, "declaration"),
        (5, 5, bytes, "parameter"),
        (5, 6, bytes - 1, "name byte"),
    ] {
        assert!(
            check([inventory], declarations, parameters, names)
                .unwrap_err()
                .contains(message)
        );
    }
    check([inventory, inventory], 10, 12, 2 * bytes).unwrap();
    assert!(check([inventory, inventory], 9, 12, 2 * bytes).is_err());
    check([&JavaSourceInventory::default()], 0, 0, 0).unwrap();
    assert_eq!(
        (MAX_DECLARATIONS, MAX_PARAMETERS, MAX_NAME_BYTES),
        (100_000, 1_000_000, 64 * 1024 * 1024)
    );
}
