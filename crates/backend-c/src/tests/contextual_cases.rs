//! Promoted case equality across the complete integer scalar domain.

use super::contextual_reconstruction::{fixture, int, key, package};
use super::*;

#[test]
fn duplicate_cases_are_compared_after_every_integer_promotion() {
    for scalar in CScalarType::ALL {
        if scalar == CScalarType::F64 {
            continue;
        }
        for duplicate in [true, false] {
            let (mut registry, file, function, scope) = fixture();
            let arm_scope = registry
                .register_scope(&function, Some(&scope), key("arm"))
                .unwrap();
            let default_scope = registry
                .register_scope(&function, Some(&scope), key("otherwise"))
                .unwrap();
            let switch = registry.register_switch(&scope, key("choice")).unwrap();
            let values = CExpressions::new(&registry);
            let ast = CStatements::new(&registry, function.clone()).unwrap();
            let exit = ast
                .break_statement(CBreakTarget::Switch(switch.clone()))
                .unwrap();
            let cases = vec![
                CCaseConstant::Signed(CSignedLiteral::I64(-1)),
                CCaseConstant::Unsigned(CUnsignedLiteral::U64(if duplicate {
                    u64::MAX
                } else {
                    0
                })),
            ];
            let arm = ast
                .switch_arm(
                    cases.clone(),
                    ast.block(arm_scope, vec![exit.clone()]).unwrap(),
                )
                .unwrap();
            let selection = ast
                .switch_statement(
                    switch,
                    values.numeric_conversion(scalar, int(&values)).unwrap(),
                    vec![arm],
                    ast.block(default_scope, vec![exit]).unwrap(),
                )
                .unwrap();
            if let CStatementKind::Switch { arms, .. } = selection.kind() {
                assert_eq!(arms[0].cases(), cases);
            } else {
                unreachable!()
            }
            let source = package(&registry, file, function, scope, vec![selection]);
            let result = registry.check_context(&[source]);
            if duplicate {
                assert_eq!(result, Err(CContextError::DuplicateCase), "{scalar:?}");
            } else {
                result.unwrap();
            }
        }
    }
}

#[test]
fn enumerator_case_identity_does_not_override_its_integer_value() {
    let (mut registry, file, function, scope) = fixture();
    let enumeration = registry.declare_enum(&file, key("Choice")).unwrap();
    let enumerator = registry
        .register_enumerator(&enumeration, key("Zero"), 0)
        .unwrap();
    registry
        .define_enum(&enumeration, vec![enumerator.clone()])
        .unwrap();
    let arm_scope = registry
        .register_scope(&function, Some(&scope), key("arm"))
        .unwrap();
    let default_scope = registry
        .register_scope(&function, Some(&scope), key("otherwise"))
        .unwrap();
    let switch = registry.register_switch(&scope, key("choice")).unwrap();
    let values = CExpressions::new(&registry);
    let ast = CStatements::new(&registry, function.clone()).unwrap();
    let exit = ast
        .break_statement(CBreakTarget::Switch(switch.clone()))
        .unwrap();
    let arm = ast
        .switch_arm(
            vec![
                CCaseConstant::Enumerator(enumerator),
                CCaseConstant::Unsigned(CUnsignedLiteral::U8(0)),
            ],
            ast.block(arm_scope, vec![exit.clone()]).unwrap(),
        )
        .unwrap();
    let selection = ast
        .switch_statement(
            switch,
            int(&values),
            vec![arm],
            ast.block(default_scope, vec![exit]).unwrap(),
        )
        .unwrap();
    let mut source = package(&registry, file.clone(), function, scope, vec![selection]);
    source.items.insert(
        0,
        CFileItem::Declaration(
            CDeclarations::new(&registry, file)
                .unwrap()
                .enumeration(enumeration)
                .unwrap(),
        ),
    );
    assert_eq!(
        registry.check_context(&[source]),
        Err(CContextError::DuplicateCase)
    );
}
