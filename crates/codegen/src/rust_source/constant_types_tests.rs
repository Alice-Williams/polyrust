//! Exact value/kind identity, constant-only detection and combined fact limits.
use super::*;

fn id(crate_id: u64, definition_path_hash: u64) -> RustDeclarationId {
    RustDeclarationId {
        crate_id,
        definition_path_hash,
    }
}
fn empty() -> RustSourceTypes {
    RustSourceTypes::new(id(1, 0), BTreeMap::new(), BTreeMap::new()).unwrap()
}

#[test]
fn constant_only_characters_keep_original_kind_and_exact_value() {
    let values = [
        RustConstantValue::Char('\0'),
        RustConstantValue::Char('\u{10ffff}'),
        RustConstantValue::I32(0x10ffff),
        RustConstantValue::I64(i64::MIN),
        RustConstantValue::Bool(true),
        RustConstantValue::F64Bits(0),
        RustConstantValue::F64Bits(1 << 63),
        RustConstantValue::F64Bits(0x7ff0000000000000),
    ];
    let map: BTreeMap<_, _> = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| (id(1, index as u64 + 1), value))
        .collect();
    let facts = empty().with_constants(map.clone()).unwrap();
    assert_eq!(facts.constants(), &map);
    assert!(facts.contains_char());
    assert_ne!(values[1], values[2]);
    assert_ne!(values[5], values[6]);
    for value in values {
        let facts = empty()
            .with_constants(BTreeMap::from([(id(1, 1), value)]))
            .unwrap();
        assert_eq!(
            facts.contains_char(),
            matches!(value, RustConstantValue::Char(_))
        );
    }
    assert!(
        !facts
            .with_constants(BTreeMap::new())
            .unwrap()
            .contains_char()
    );
}

#[test]
fn constant_owner_and_other_declaration_roles_cannot_conflict() {
    let facts = RustSourceTypes::new(
        id(1, 0),
        BTreeMap::from([(
            id(1, 1),
            RustFunctionTypes {
                parameters: vec![],
                result: RustResultKind::Unit,
            },
        )]),
        BTreeMap::from([(
            id(1, 2),
            RustFieldTypes {
                owner: id(1, 3),
                kind: RustScalarKind::Char,
            },
        )]),
    )
    .unwrap();
    for invalid in [id(2, 4), id(1, 0), id(1, 1), id(1, 2), id(1, 3)] {
        assert!(
            facts
                .clone()
                .with_constants(BTreeMap::from([(invalid, RustConstantValue::Char('a'))]))
                .is_err(),
            "{invalid:?}"
        );
    }
    facts
        .with_constants(BTreeMap::from([(id(1, 4), RustConstantValue::Char('a'))]))
        .unwrap();
}

#[test]
fn constants_share_scalar_budget_with_function_parameters_and_fields() {
    let facts = RustSourceTypes::new(
        id(1, 0),
        BTreeMap::from([(
            id(1, 1),
            RustFunctionTypes {
                parameters: vec![RustScalarKind::I32; 99_997],
                result: RustResultKind::Unit,
            },
        )]),
        BTreeMap::from([(
            id(1, 2),
            RustFieldTypes {
                owner: id(1, 3),
                kind: RustScalarKind::I32,
            },
        )]),
    )
    .unwrap();
    let constants = BTreeMap::from([(id(1, 4), RustConstantValue::Char('a'))]);
    facts.clone().with_constants(constants.clone()).unwrap();
    let mut extra = constants;
    extra.insert(id(1, 5), RustConstantValue::Bool(false));
    assert!(
        facts
            .with_constants(extra)
            .unwrap_err()
            .contains("scalar budget")
    );
    let many: BTreeMap<_, _> = (1..=100_000)
        .map(|index| (id(1, index), RustConstantValue::I32(0)))
        .collect();
    empty().with_constants(many.clone()).unwrap();
    let mut extra = many;
    extra.insert(id(1, 100_001), RustConstantValue::I32(0));
    assert!(
        empty()
            .with_constants(extra)
            .unwrap_err()
            .contains("declaration budget")
    );
}
