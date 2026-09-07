use super::{
    DiagnosticCode, JavaBlock, JavaDialect, JavaExpr, JavaExprKind, JavaField, JavaIdentifier,
    JavaKnownType, JavaMember, JavaMethod, JavaMethodDeclaration, JavaModifier, JavaPrecedence,
    JavaPrimitive, JavaStmt, JavaType, fixture_declaration, parameter, structural_method,
    verify_fixture,
};

#[test]
fn contextual_types_require_exact_known_arity_and_declared_variables() {
    let invalid_arity = fixture_declaration(vec![JavaMember::Field(JavaField {
        declared: None,
        modifiers: vec![JavaModifier::Static],
        ty: JavaType::generic(
            JavaKnownType::List,
            vec![
                JavaType::known(JavaKnownType::String),
                JavaType::known(JavaKnownType::Object),
            ],
        ),
        name: JavaIdentifier::from_portable("values"),
        initializer: None,
    })]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], invalid_arity)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch
            && value.message.contains("exactly 1 type arguments")
    }));

    let variable = JavaIdentifier::from_portable("T");
    let undeclared = fixture_declaration(vec![structural_method(
        "identity",
        JavaType::TypeVariable(variable.clone()),
        vec![parameter(JavaType::TypeVariable(variable.clone()), "value")],
        JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::local(
            JavaType::TypeVariable(variable.clone()),
            JavaIdentifier::from_portable("value"),
        )))]),
    )]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], undeclared)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::UnresolvedReference && value.message.contains("type variable")
    }));

    let declared = fixture_declaration(vec![JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Public, JavaModifier::Static],
        type_parameters: vec![variable.clone()],
        return_type: JavaType::TypeVariable(variable.clone()),
        name: JavaIdentifier::from_portable("identity"),
        parameters: vec![parameter(JavaType::TypeVariable(variable.clone()), "value")],
        body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
            JavaExpr::local(
                JavaType::TypeVariable(variable),
                JavaIdentifier::from_portable("value"),
            ),
        ))])),
    })]);
    assert!(
        verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], declared)],
        )
        .is_ok()
    );
}

#[test]
fn casts_and_instanceof_require_java_legality_and_reifiable_targets() {
    let string = JavaType::known(JavaKnownType::String);
    let object = JavaType::known(JavaKnownType::Object);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let invalid_cast = JavaExpr {
        ty: int.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: int.clone(),
            value: Box::new(JavaExpr::local(
                string.clone(),
                JavaIdentifier::from_portable("value"),
            )),
        },
    };
    let invalid_primitive_test = JavaExpr {
        ty: boolean.clone(),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::InstanceOf {
            value: Box::new(JavaExpr::local(
                int.clone(),
                JavaIdentifier::from_portable("number"),
            )),
            target: string.clone(),
            binding: None,
        },
    };
    let non_reifiable = JavaType::generic(JavaKnownType::List, vec![string.clone()]);
    let invalid_generic_test = JavaExpr {
        ty: boolean.clone(),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::InstanceOf {
            value: Box::new(JavaExpr::local(
                object.clone(),
                JavaIdentifier::from_portable("value"),
            )),
            target: non_reifiable.clone(),
            binding: None,
        },
    };
    let boxed_long = JavaType::primitive(JavaPrimitive::Long).boxed();
    let invalid_boxed_cast = JavaExpr {
        ty: boxed_long.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: boxed_long.clone(),
            value: Box::new(JavaExpr::local(
                int.clone(),
                JavaIdentifier::from_portable("boxedNumber"),
            )),
        },
    };
    let invalid = fixture_declaration(vec![
        structural_method(
            "cast",
            int.clone(),
            vec![parameter(string, "value")],
            JavaBlock::new(vec![JavaStmt::Return(Some(invalid_cast))]),
        ),
        structural_method(
            "primitiveTest",
            boolean.clone(),
            vec![parameter(int.clone(), "number")],
            JavaBlock::new(vec![JavaStmt::Return(Some(invalid_primitive_test))]),
        ),
        structural_method(
            "genericTest",
            boolean.clone(),
            vec![parameter(object.clone(), "value")],
            JavaBlock::new(vec![JavaStmt::Return(Some(invalid_generic_test))]),
        ),
        structural_method(
            "boxedCast",
            boxed_long,
            vec![parameter(int.clone(), "boxedNumber")],
            JavaBlock::new(vec![JavaStmt::Return(Some(invalid_boxed_cast))]),
        ),
    ]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], invalid)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch && value.message.contains("cast is not legal")
    }));
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch
            && value.message.contains("instanceof is not legal")
    }));
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch
            && value
                .message
                .contains("instanceof target must be reifiable")
    }));

    let unchecked_cast = JavaExpr {
        ty: non_reifiable.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: non_reifiable.clone(),
            value: Box::new(JavaExpr::local(
                object.clone(),
                JavaIdentifier::from_portable("value"),
            )),
        },
    };
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(
            vec![],
            fixture_declaration(vec![structural_method(
                "uncheckedCast",
                non_reifiable,
                vec![parameter(object.clone(), "value")],
                JavaBlock::new(vec![JavaStmt::Return(Some(unchecked_cast))]),
            )]),
        )],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch && value.message.contains("cast is not legal")
    }));

    let redundant_cast = JavaExpr {
        ty: int.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Cast {
            target: int.clone(),
            value: Box::new(JavaExpr::local(
                int.clone(),
                JavaIdentifier::from_portable("value"),
            )),
        },
    };
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(
            vec![],
            fixture_declaration(vec![structural_method(
                "redundantCast",
                int.clone(),
                vec![parameter(int, "value")],
                JavaBlock::new(vec![JavaStmt::Return(Some(redundant_cast))]),
            )]),
        )],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch && value.message.contains("cast is redundant")
    }));

    let list_any = JavaType::generic(
        JavaKnownType::List,
        vec![JavaType::Wildcard { bound: None }],
    );
    let valid = fixture_declaration(vec![structural_method(
        "listTest",
        boolean,
        vec![parameter(object.clone(), "value")],
        JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: JavaType::primitive(JavaPrimitive::Boolean),
            precedence: JavaPrecedence::Relational,
            kind: JavaExprKind::InstanceOf {
                value: Box::new(JavaExpr::local(
                    object,
                    JavaIdentifier::from_portable("value"),
                )),
                target: list_any,
                binding: Some(JavaIdentifier::from_portable("values")),
            },
        }))]),
    )]);
    assert!(
        verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], valid)],
        )
        .is_ok()
    );
}
