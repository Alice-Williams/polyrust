//! Cross-arm/default and exact label occurrence controls.
use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

#[test]
fn promoted_case_duplicates_cross_arm_boundaries_and_default_requires_an_exit() {
    for duplicate in [false, true] {
        for default_exits in [false, true] {
            let (mut registry, file, function, scope) = fixture();
            let scopes = ["first", "second", "otherwise"].map(|name| {
                registry
                    .register_scope(&function, Some(&scope), key(name))
                    .unwrap()
            });
            let switch = registry.register_switch(&scope, key("choice")).unwrap();
            let values = CExpressions::new(&registry);
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let exit = ast
                .break_statement(CBreakTarget::Switch(switch.clone()))
                .unwrap();
            let cases = [
                CCaseConstant::Signed(CSignedLiteral::I64(-1)),
                CCaseConstant::Unsigned(CUnsignedLiteral::U64(if duplicate {
                    u64::MAX
                } else {
                    0
                })),
            ];
            let arms = cases
                .into_iter()
                .zip(scopes[..2].iter())
                .map(|(case, scope)| {
                    ast.switch_arm(
                        vec![case],
                        ast.block(scope.clone(), vec![exit.clone()]).unwrap(),
                    )
                    .unwrap()
                })
                .collect();
            let selection = ast
                .switch_statement(
                    switch,
                    int(&values),
                    arms,
                    ast.block(
                        scopes[2].clone(),
                        if default_exits { vec![exit] } else { vec![] },
                    )
                    .unwrap(),
                )
                .unwrap();
            let source = package(&registry, file, function, scope, vec![selection]);
            assert_eq!(
                registry.check_context(&[source]),
                if duplicate {
                    Err(CContextError::DuplicateCase)
                } else if default_exits {
                    Ok(())
                } else {
                    Err(CContextError::SwitchFallthrough)
                }
            );
        }
    }
}

#[test]
fn cleanup_label_occurrences_cannot_be_deleted_or_duplicated_after_construction() {
    let (mut registry, file, function, scope) = fixture();
    let label = registry
        .register_cleanup_exit(&scope, key("cleanup"))
        .unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let source = package(
        &registry,
        file,
        function,
        scope,
        vec![
            ast.label(label, ast.discard(int(&values)).unwrap())
                .unwrap(),
        ],
    );
    registry
        .check_context(std::slice::from_ref(&source))
        .unwrap();
    for duplicate in [false, true] {
        let mut bad = source.clone();
        let CFileItem::Definition(definition) = &mut bad.items[0] else {
            unreachable!()
        };
        let CDefinitionKind::Function { body, .. } = &mut definition.kind else {
            unreachable!()
        };
        if duplicate {
            body.statements.push(body.statements[0].clone());
        } else {
            body.statements.clear();
        }
        assert_eq!(
            registry.check_context(&[bad]),
            if duplicate {
                Err(CContextError::DuplicateOccurrence)
            } else {
                Err(CContextError::MissingRegistrationOccurrence)
            }
        );
    }
}
