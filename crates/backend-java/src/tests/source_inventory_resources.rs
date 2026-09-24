use super::*;
use crate::tests::source_dependency_fixture::*;

#[test]
fn canonical_owner_inventory_charges_six_constants_and_four_types() {
    let ready = crate::tests::canonical_owner_resources::package();
    let inventory = &ready.ast().files()[0].items()[0].source_inventory;
    assert_eq!(inventory.iter().len(), 10);
    let bytes = [
        "Generated",
        "Outcome",
        "Success",
        "Error",
        "EMPTY",
        "INVALID_DIGIT",
        "POS_OVERFLOW",
        "NEG_OVERFLOW",
        "ZERO",
        "NOT_A_POWER_OF_TWO",
    ]
    .iter()
    .map(|name| name.len())
    .sum();
    check([inventory], 10, 0, bytes).unwrap();
    assert!(
        check([inventory], 9, 0, bytes)
            .unwrap_err()
            .contains("declaration")
    );
    assert!(
        check([inventory], 10, 0, bytes - 1)
            .unwrap_err()
            .contains("name byte")
    );
    let exact = || std::iter::repeat_n(inventory, MAX_DECLARATIONS / 10);
    check(exact(), MAX_DECLARATIONS, MAX_PARAMETERS, MAX_NAME_BYTES).unwrap();
    let extra = certify(package(9, vec![]));
    let extra = &extra.ast().files()[0].items()[0].source_inventory;
    assert_eq!(extra.iter().len(), 1);
    assert!(
        check(
            exact().chain([extra]),
            MAX_DECLARATIONS,
            MAX_PARAMETERS,
            MAX_NAME_BYTES
        )
        .unwrap_err()
        .contains("declaration")
    );
}

#[test]
fn result_owner_inventory_charges_all_six_adapter_types_and_the_facade() {
    let api = crate::tests::result_imports::fixture::owner();
    let inventory = &api.package().ast().files()[0].items()[0].source_inventory;
    assert_eq!(inventory.iter().len(), 7);
    let bytes = [
        "Generated",
        "Outcome",
        "Success",
        "Error",
        "OtherOutcome",
        "OtherSuccess",
        "OtherError",
    ]
    .iter()
    .map(|name| name.len())
    .sum();
    check([inventory], 7, 0, bytes).unwrap();
    assert!(
        check([inventory], 6, 0, bytes)
            .unwrap_err()
            .contains("declaration")
    );
    assert!(
        check([inventory], 7, 0, bytes - 1)
            .unwrap_err()
            .contains("name byte")
    );
}

#[test]
fn result_owner_inventory_reaches_the_production_declaration_boundary() {
    let api = crate::tests::result_imports::fixture::owner();
    let family = &api.package().ast().files()[0].items()[0].source_inventory;
    let remainder = certify(package(8, functions(42)));
    let remainder = &remainder.ast().files()[0].items()[0].source_inventory;
    let extra = certify(package(9, vec![]));
    let extra = &extra.ast().files()[0].items()[0].source_inventory;
    assert_eq!(
        (
            family.iter().len(),
            remainder.iter().len(),
            extra.iter().len()
        ),
        (7, 5, 1)
    );
    let copies = MAX_DECLARATIONS / 7;
    assert_eq!(copies * 7 + 5, MAX_DECLARATIONS);
    // Exercise the production streaming counter with authenticated inventories;
    // repeated inputs are charged, not silently deduplicated by owner identity.
    let exact = || std::iter::repeat_n(family, copies).chain([remainder]);
    check(exact(), MAX_DECLARATIONS, MAX_PARAMETERS, MAX_NAME_BYTES).unwrap();
    assert!(
        check(
            exact().chain([extra]),
            MAX_DECLARATIONS,
            MAX_PARAMETERS,
            MAX_NAME_BYTES
        )
        .unwrap_err()
        .contains("declaration limit")
    );
}

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
