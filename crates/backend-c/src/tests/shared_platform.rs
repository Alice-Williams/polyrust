//! Independent platform inventory, structural formatting and mutation controls.
use super::{CDialect, platform, project_c_package, tests::fixture};
use crate::ast::*;

pub(super) fn expected() -> String {
    let mut text = String::new();
    for (name, ty, bytes) in [
        ("Bool", "_Bool", 1),
        ("Int", "int", 4),
        ("I32", "int32_t", 4),
        ("Size", "size_t", 8),
        ("Pointer", "void *", 8),
    ] {
        for (query, operator) in [("Size", "sizeof"), ("Alignment", "_Alignof")] {
            text.push_str(&format!(
                "_Static_assert(({operator}({ty}) == {bytes}UL), \"C profile {name} {query}\");\n\n"
            ));
        }
    }
    text
}

pub(super) fn installed() -> (CFrozenRegistry, CSourceFile) {
    let (registry, source) = fixture(CScalarType::I32);
    let package = project_c_package(registry, vec![source]).unwrap();
    let unit = &package.files().next().unwrap().items()[0];
    (
        unit.projection.registry.clone(),
        unit.data.source.as_ref().clone(),
    )
}

#[test]
fn required_platform_checks_are_typed_exact_and_idempotent() {
    let (registry, source) = installed();
    assert_eq!(
        source
            .items()
            .iter()
            .filter(|item| matches!(item, CFileItem::StaticAssert(_)))
            .count(),
        10
    );
    platform::verify(&source).unwrap();
    let twice = platform::install(&registry, source.clone()).unwrap();
    assert_eq!(twice, source);
    let package = project_c_package(registry, vec![twice]).unwrap();
    portable_codegen::verify_unresolved_package(&CDialect, package).unwrap();
}

#[test]
fn i64_checks_are_required_by_source_types_not_by_their_own_assertions() {
    let (registry, source) = fixture(CScalarType::I64);
    let wide = platform::install(&registry, source).unwrap();
    platform::verify(&wide).unwrap();
    assert_eq!(wide, platform::install(&registry, wide.clone()).unwrap());
    let is_wide_check = |item: &CFileItem| {
        matches!(item, CFileItem::StaticAssert(assertion)
            if platform::classify(assertion).unwrap().object == platform::Object::I64)
    };
    let checks = wide
        .items()
        .iter()
        .filter(|item| is_wide_check(item))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(checks.len(), 2);
    assert_eq!(
        wide.items()
            .iter()
            .filter(|item| matches!(item, CFileItem::StaticAssert(_)))
            .count(),
        12
    );
    let declarations =
        CDeclarations::new(registry.registrations(), wide.identity().clone()).unwrap();
    for missing in &checks {
        let items = wide
            .items()
            .iter()
            .filter(|item| *item != missing)
            .cloned()
            .collect();
        let incomplete = declarations.source_file(items).unwrap();
        assert!(project_c_package(registry.clone(), vec![incomplete]).is_err());
    }
    let (narrow_registry, narrow) = installed();
    let declarations =
        CDeclarations::new(narrow_registry.registrations(), narrow.identity().clone()).unwrap();
    let mut items = narrow.items().to_vec();
    let expressions = CExpressions::new(narrow_registry.registrations());
    for query in [
        expressions.size_of(CObjectType::scalar(CScalarType::I64)),
        expressions.align_of(CObjectType::scalar(CScalarType::I64)),
    ] {
        let condition = expressions
            .binary(
                CBinaryOperator::Equal,
                query.unwrap(),
                expressions
                    .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(8)))
                    .unwrap(),
            )
            .unwrap();
        items.push(CFileItem::StaticAssert(
            declarations
                .static_assert(
                    condition,
                    CAssertDiagnostic::new(b"extra i64 check".to_vec()),
                )
                .unwrap(),
        ));
    }
    let self_authorizing = declarations.source_file(items).unwrap();
    assert!(project_c_package(narrow_registry, vec![self_authorizing]).is_err());
}

#[test]
fn missing_duplicate_and_incorrect_platform_requirements_are_rejected() {
    for mutation in 0..3 {
        let (registry, source) = installed();
        let declarations =
            CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
        let mut items = source.items().to_vec();
        match mutation {
            0 => {
                items.remove(0);
            }
            1 => items.insert(0, items[0].clone()),
            2 => {
                let expressions = CExpressions::new(registry.registrations());
                let wrong = expressions
                    .binary(
                        CBinaryOperator::Equal,
                        expressions
                            .size_of(CObjectType::scalar(CScalarType::Bool))
                            .unwrap(),
                        expressions
                            .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(2)))
                            .unwrap(),
                    )
                    .unwrap();
                items[0] = CFileItem::StaticAssert(
                    declarations
                        .static_assert(wrong, CAssertDiagnostic::new(b"wrong".to_vec()))
                        .unwrap(),
                );
            }
            _ => unreachable!(),
        }
        let source = declarations.source_file(items).unwrap();
        assert!(project_c_package(registry, vec![source]).is_err());
    }
}
