use super::{
    DiagnosticCode, JavaArrayOwnership, JavaArrayOwnershipTransition, JavaBlock, JavaDialect,
    JavaExpr, JavaExprKind, JavaIdentifier, JavaLiteral, JavaLocalFinality, JavaPrecedence,
    JavaPrimitive, JavaStmt, JavaType, fixture_declaration, parameter, structural_method,
    verify_fixture,
};

#[test]
fn array_ownership_transitions_require_a_fresh_internal_copy() {
    let byte = JavaType::primitive(JavaPrimitive::Byte);
    let internal = JavaType::Array {
        component: Box::new(byte.clone()),
        ownership: JavaArrayOwnership::InternalMutable,
    };
    let boundary = JavaType::Array {
        component: Box::new(byte.clone()),
        ownership: JavaArrayOwnership::DefensiveCopyBoundary,
    };
    let fresh = JavaExpr {
        ty: internal.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::NewArray {
            component: byte,
            length: Box::new(JavaExpr::literal(
                JavaType::primitive(JavaPrimitive::Int),
                JavaLiteral::I32(1),
            )),
        },
    };
    let promoted = JavaExpr {
        ty: boundary.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::ArrayOwnershipTransition {
            transition: JavaArrayOwnershipTransition::FreshCopyToBoundary,
            value: Box::new(fresh),
        },
    };
    assert!(
        verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(
                vec![],
                fixture_declaration(vec![structural_method(
                    "validCopy",
                    boundary.clone(),
                    vec![],
                    JavaBlock::new(vec![JavaStmt::Return(Some(promoted))]),
                )]),
            )],
        )
        .is_ok()
    );

    let invalid = JavaExpr {
        ty: boundary.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::ArrayOwnershipTransition {
            transition: JavaArrayOwnershipTransition::FreshCopyToBoundary,
            value: Box::new(JavaExpr::local(
                boundary.clone(),
                JavaIdentifier::from_portable("value"),
            )),
        },
    };
    assert!(
        verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(
                vec![],
                fixture_declaration(vec![structural_method(
                    "invalidCopy",
                    boundary.clone(),
                    vec![parameter(boundary.clone(), "value")],
                    JavaBlock::new(vec![JavaStmt::Return(Some(invalid))]),
                )]),
            )],
        )
        .is_err()
    );

    let forged_cast = JavaExpr {
        ty: internal.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: internal.clone(),
            value: Box::new(JavaExpr::local(
                boundary.clone(),
                JavaIdentifier::from_portable("value"),
            )),
        },
    };
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(
            vec![],
            fixture_declaration(vec![structural_method(
                "forgedOwnershipCast",
                JavaType::primitive(JavaPrimitive::Void),
                vec![parameter(boundary, "value")],
                JavaBlock::new(vec![JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: internal,
                    name: JavaIdentifier::from_portable("internal"),
                    value: Some(forged_cast),
                }]),
            )]),
        )],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch && value.message.contains("cast is not legal")
    }));
}
