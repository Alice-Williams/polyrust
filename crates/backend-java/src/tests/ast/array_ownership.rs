use super::{
    DiagnosticCode, JavaArrayOwnership, JavaArrayOwnershipTransition, JavaBlock, JavaDialect,
    JavaExpr, JavaExprKind, JavaIdentifier, JavaLiteral, JavaLocalFinality, JavaPrecedence,
    JavaPrimitive, JavaStmt, JavaType, fixture_declaration, parameter, structural_method,
    verify_fixture,
};

#[test]
fn array_ownership_erasure_checks_nested_wrappers_and_both_directions() {
    use super::super::invocations::value_transfer_preserves_isolation;
    let object = JavaType::known(super::JavaKnownType::Object);
    let internal = JavaType::Array {
        component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
        ownership: JavaArrayOwnership::InternalMutable,
    };
    let nested = [
        internal.clone(),
        JavaType::generic(super::JavaKnownType::List, vec![internal.clone()]),
        JavaType::Wildcard {
            bound: Some((
                super::JavaWildcardBound::Extends,
                Box::new(internal.clone()),
            )),
        },
        JavaType::Array {
            component: Box::new(internal),
            ownership: JavaArrayOwnership::DefensiveCopyBoundary,
        },
    ];
    for ty in nested {
        assert!(!value_transfer_preserves_isolation(&ty, &object));
        assert!(!value_transfer_preserves_isolation(&object, &ty));
        assert!(value_transfer_preserves_isolation(&ty, &ty));
    }
    let copied = JavaType::Array {
        component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
        ownership: JavaArrayOwnership::DefensiveCopyBoundary,
    };
    assert!(value_transfer_preserves_isolation(&copied, &object));
}

#[test]
fn array_ownership_cannot_be_erased_by_object_cast() {
    let byte = JavaType::primitive(JavaPrimitive::Byte);
    let object = JavaType::known(super::JavaKnownType::Object);
    let fresh = JavaExpr {
        ty: JavaType::Array {
            component: Box::new(byte.clone()),
            ownership: JavaArrayOwnership::InternalMutable,
        },
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::NewArray {
            component: byte,
            length: Box::new(JavaExpr::literal(
                JavaType::primitive(JavaPrimitive::Int),
                JavaLiteral::I32(1),
            )),
        },
    };
    let erased = JavaExpr {
        ty: object.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: object.clone(),
            value: Box::new(fresh.clone()),
        },
    };
    let fallback = JavaExpr {
        ty: object.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: object.clone(),
            value: Box::new(JavaExpr::literal(
                JavaType::known(super::JavaKnownType::String),
                JavaLiteral::String("fallback".to_owned()),
            )),
        },
    };
    let patterned = JavaStmt::If {
        condition: JavaExpr {
            ty: JavaType::primitive(JavaPrimitive::Boolean),
            precedence: JavaPrecedence::Relational,
            kind: JavaExprKind::InstanceOf {
                value: Box::new(fresh),
                target: object.clone(),
                binding: Some(JavaIdentifier::new("matched").unwrap()),
            },
        },
        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::local(
            object.clone(),
            JavaIdentifier::new("matched").unwrap(),
        )))]),
        else_block: Some(JavaBlock::new(vec![JavaStmt::Return(Some(fallback))])),
    };
    let results = [JavaStmt::Return(Some(erased)), patterned]
        .into_iter()
        .map(|statement| {
            verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(
                    vec![],
                    fixture_declaration(vec![structural_method(
                        "leak",
                        object.clone(),
                        vec![],
                        JavaBlock::new(vec![statement]),
                    )]),
                )],
            )
        })
        .collect::<Vec<_>>();
    assert!(
        results.iter().all(Result::is_err),
        "array erasure (cast, pattern): {results:?}"
    );
}

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
