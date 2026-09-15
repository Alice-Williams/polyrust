//! Implementation-only wide values still impose header-owned ABI obligations.
use super::{CDialect, package_fixture, platform, project_c_package};
use crate::{ast::*, dialect::dependencies::source_scalars};
use portable_codegen::{TargetLinker, certify_resolved_package, verify_unresolved_package};

#[test]
fn private_wide_comparison_requires_both_header_checks_without_a_wide_export() {
    let mut fixture = package_fixture::fixture();
    let declarations = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[1].identity().clone(),
    )
    .unwrap();
    let expressions = CExpressions::new(fixture.registry.registrations());
    let wide = expressions
        .literal(CLiteral::Signed(CSignedLiteral::I64(i64::MIN)))
        .unwrap();
    let comparison = expressions
        .binary(CBinaryOperator::Less, wide.clone(), wide)
        .unwrap();
    let mut items = fixture.files[1].items().to_vec();
    let mut changed = false;
    for item in &mut items {
        let CFileItem::Definition(definition) = item else {
            continue;
        };
        let CDefinitionKind::Function {
            function,
            linkage,
            parameters,
            body,
        } = definition.kind()
        else {
            continue;
        };
        if function != &fixture.helper {
            continue;
        }
        let statements =
            CStatements::new(fixture.registry.registrations(), function.clone()).unwrap();
        let mut body_items = vec![statements.discard(comparison.clone()).unwrap()];
        body_items.extend_from_slice(body.statements());
        let body = statements.block(body.scope().clone(), body_items).unwrap();
        *item = CFileItem::Definition(
            declarations
                .function_definition(function.clone(), *linkage, parameters.clone(), body)
                .unwrap(),
        );
        assert!(!changed);
        changed = true;
    }
    assert!(changed);
    fixture.files[1] = declarations.source_file(items).unwrap();
    assert!(!source_scalars(&fixture.files[..1]).contains(&CScalarType::I64));
    assert!(source_scalars(&fixture.files[1..]).contains(&CScalarType::I64));
    let installed = platform::install_package(&fixture.registry, fixture.files.clone()).unwrap();
    let is_wide = |item: &CFileItem| {
        matches!(item, CFileItem::StaticAssert(assertion)
        if platform::classify(assertion).unwrap().object == platform::Object::I64)
    };
    assert_eq!(
        installed[0]
            .items()
            .iter()
            .filter(|item| is_wide(item))
            .count(),
        2
    );
    assert!(
        !installed[1]
            .items()
            .iter()
            .any(|item| matches!(item, CFileItem::StaticAssert(_)))
    );
    platform::verify_package(&installed).unwrap();
    let draft = project_c_package(fixture.registry.clone(), installed.clone()).unwrap();
    let checked = verify_unresolved_package(&CDialect, draft).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    certify_resolved_package(&CDialect, linked).unwrap();
    for (index, _) in installed[0]
        .items()
        .iter()
        .enumerate()
        .filter(|(_, item)| is_wide(item))
    {
        let mut changed = installed.clone();
        let mut header_items = changed[0].items().to_vec();
        header_items.remove(index);
        let declarations = CDeclarations::new(
            fixture.registry.registrations(),
            changed[0].identity().clone(),
        )
        .unwrap();
        changed[0] = declarations.source_file(header_items).unwrap();
        assert!(project_c_package(fixture.registry.clone(), changed).is_err());
    }
}
