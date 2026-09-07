use super::{
    DiagnosticCode, JavaBinaryOperator, JavaBlock, JavaCatch, JavaDialect, JavaExpr, JavaExprKind,
    JavaIdentifier, JavaKnownType, JavaLocalFinality, JavaPattern, JavaPrecedence, JavaPrimitive,
    JavaStmt, JavaSwitchArm, JavaType, fixture_declaration, instanceof, parameter,
    structural_method, verify_fixture,
};

#[test]
fn nested_bindings_reject_outer_collisions_and_never_replace_the_outer_binding() {
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let object = JavaType::known(JavaKnownType::Object);
    let string = JavaType::known(JavaKnownType::String);
    let runtime_exception = JavaType::known(JavaKnownType::RuntimeException);
    let strings = JavaType::generic(JavaKnownType::List, vec![string.clone()]);
    let preserve_outer_string = |name: &str| JavaStmt::Local {
        finality: JavaLocalFinality::Final,
        ty: string.clone(),
        name: JavaIdentifier::from_portable("preserved"),
        value: Some(JavaExpr::local(
            string.clone(),
            JavaIdentifier::from_portable(name),
        )),
    };
    let preserve_outer_object = |name: &str| JavaStmt::Local {
        finality: JavaLocalFinality::Final,
        ty: object.clone(),
        name: JavaIdentifier::from_portable("preserved"),
        value: Some(JavaExpr::local(
            object.clone(),
            JavaIdentifier::from_portable(name),
        )),
    };

    let declaration = fixture_declaration(vec![
        structural_method(
            "foreachCollision",
            JavaType::primitive(JavaPrimitive::Void),
            vec![
                parameter(strings.clone(), "values"),
                parameter(string.clone(), "item"),
            ],
            JavaBlock::new(vec![JavaStmt::ForEach {
                binding_type: string.clone(),
                binding: JavaIdentifier::from_portable("item"),
                iterable: JavaExpr::local(strings, JavaIdentifier::from_portable("values")),
                body: JavaBlock::new(vec![preserve_outer_string("item")]),
            }]),
        ),
        structural_method(
            "switchCollision",
            JavaType::primitive(JavaPrimitive::Void),
            vec![parameter(object.clone(), "selector")],
            JavaBlock::new(vec![JavaStmt::Switch {
                value: JavaExpr::local(object.clone(), JavaIdentifier::from_portable("selector")),
                arms: vec![
                    JavaSwitchArm {
                        pattern: JavaPattern::Type {
                            ty: string.clone(),
                            binding: JavaIdentifier::from_portable("selector"),
                        },
                        body: JavaBlock::new(vec![preserve_outer_object("selector")]),
                    },
                    JavaSwitchArm {
                        pattern: JavaPattern::Default,
                        body: JavaBlock::new(vec![]),
                    },
                ],
            }]),
        ),
        structural_method(
            "catchCollision",
            JavaType::primitive(JavaPrimitive::Void),
            vec![parameter(string.clone(), "failure")],
            JavaBlock::new(vec![JavaStmt::TryCatch {
                try_block: JavaBlock::new(vec![]),
                catches: vec![JavaCatch {
                    exception_type: runtime_exception,
                    binding: JavaIdentifier::from_portable("failure"),
                    body: JavaBlock::new(vec![preserve_outer_string("failure")]),
                }],
            }]),
        ),
        structural_method(
            "outerFlowCollision",
            JavaType::primitive(JavaPrimitive::Void),
            vec![
                parameter(object.clone(), "input"),
                parameter(string.clone(), "text"),
            ],
            JavaBlock::new(vec![JavaStmt::If {
                condition: instanceof(
                    JavaExpr::local(object.clone(), JavaIdentifier::from_portable("input")),
                    string.clone(),
                    "text",
                ),
                then_block: JavaBlock::new(vec![preserve_outer_string("text")]),
                else_block: None,
            }]),
        ),
        structural_method(
            "duplicateFlowCollision",
            JavaType::primitive(JavaPrimitive::Void),
            vec![
                parameter(object.clone(), "left"),
                parameter(object.clone(), "right"),
            ],
            JavaBlock::new(vec![JavaStmt::If {
                condition: JavaExpr {
                    ty: boolean,
                    precedence: JavaPrecedence::LogicalAnd,
                    kind: JavaExprKind::Binary {
                        operator: JavaBinaryOperator::LogicalAnd,
                        left: Box::new(instanceof(
                            JavaExpr::local(object.clone(), JavaIdentifier::from_portable("left")),
                            string.clone(),
                            "text",
                        )),
                        right: Box::new(instanceof(
                            JavaExpr::local(object, JavaIdentifier::from_portable("right")),
                            string,
                            "text",
                        )),
                    },
                },
                then_block: JavaBlock::new(vec![]),
                else_block: None,
            }]),
        ),
    ]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], declaration)],
    )
    .unwrap_err();
    let duplicates = diagnostics
        .iter()
        .filter(|value| value.code == DiagnosticCode::DuplicateDeclaration)
        .count();
    assert_eq!(duplicates, 5, "{diagnostics:?}");
    assert!(
        diagnostics
            .iter()
            .all(|value| value.code != DiagnosticCode::TypeMismatch),
        "a rejected nested binding replaced its authoritative outer binding: {diagnostics:?}"
    );
}

#[test]
fn instanceof_bindings_cannot_bypass_collision_checks_in_value_expressions() {
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let object = JavaType::known(JavaKnownType::Object);
    let string = JavaType::known(JavaKnownType::String);
    let collision = || {
        instanceof(
            JavaExpr::local(object.clone(), JavaIdentifier::from_portable("input")),
            string.clone(),
            "text",
        )
    };
    let parameters = vec![
        parameter(object.clone(), "input"),
        parameter(string.clone(), "text"),
    ];
    let declaration = fixture_declaration(vec![
        structural_method(
            "returnCollision",
            boolean.clone(),
            parameters.clone(),
            JavaBlock::new(vec![JavaStmt::Return(Some(collision()))]),
        ),
        structural_method(
            "initializerCollision",
            JavaType::primitive(JavaPrimitive::Void),
            parameters,
            JavaBlock::new(vec![JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: boolean,
                name: JavaIdentifier::from_portable("matched"),
                value: Some(collision()),
            }]),
        ),
    ]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], declaration)],
    )
    .unwrap_err();
    let collisions = diagnostics
        .iter()
        .filter(|value| value.code == DiagnosticCode::DuplicateDeclaration)
        .count();
    assert_eq!(collisions, 2, "{diagnostics:?}");
}
