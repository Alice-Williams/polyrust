use super::{
    DiagnosticCode, JavaBlock, JavaConstructorRef, JavaDialect, JavaExpr, JavaExprKind,
    JavaIdentifier, JavaKnownConstructor, JavaKnownType, JavaLiteral, JavaPattern, JavaPrecedence,
    JavaPrimitive, JavaStmt, JavaSwitchArm, JavaType, fixture_declaration, parameter,
    structural_method, verify_fixture,
};

#[test]
fn statement_expressions_and_switch_patterns_fail_closed() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let object = JavaType::known(JavaKnownType::Object);
    let string = JavaType::known(JavaKnownType::String);
    let declaration = fixture_declaration(vec![
        structural_method(
            "badStatement",
            JavaType::primitive(JavaPrimitive::Void),
            vec![],
            JavaBlock::new(vec![JavaStmt::Expression(JavaExpr::literal(
                int.clone(),
                JavaLiteral::I32(1),
            ))]),
        ),
        structural_method(
            "badConstants",
            JavaType::primitive(JavaPrimitive::Void),
            vec![parameter(int.clone(), "selector")],
            JavaBlock::new(vec![JavaStmt::Switch {
                value: JavaExpr::local(int.clone(), JavaIdentifier::from_portable("selector")),
                arms: vec![
                    JavaSwitchArm {
                        pattern: JavaPattern::Literal(JavaLiteral::I32(1)),
                        body: JavaBlock::new(vec![]),
                    },
                    JavaSwitchArm {
                        pattern: JavaPattern::Literal(JavaLiteral::CharScalar(1)),
                        body: JavaBlock::new(vec![]),
                    },
                    JavaSwitchArm {
                        pattern: JavaPattern::Literal(JavaLiteral::String("one".to_owned())),
                        body: JavaBlock::new(vec![]),
                    },
                    JavaSwitchArm {
                        pattern: JavaPattern::Default,
                        body: JavaBlock::new(vec![]),
                    },
                ],
            }]),
        ),
        structural_method(
            "badDominance",
            JavaType::primitive(JavaPrimitive::Void),
            vec![parameter(object.clone(), "selector")],
            JavaBlock::new(vec![JavaStmt::Switch {
                value: JavaExpr::local(object, JavaIdentifier::from_portable("selector")),
                arms: vec![
                    JavaSwitchArm {
                        pattern: JavaPattern::Type {
                            ty: JavaType::known(JavaKnownType::Object),
                            binding: JavaIdentifier::from_portable("anything"),
                        },
                        body: JavaBlock::new(vec![]),
                    },
                    JavaSwitchArm {
                        pattern: JavaPattern::Type {
                            ty: string,
                            binding: JavaIdentifier::from_portable("text"),
                        },
                        body: JavaBlock::new(vec![]),
                    },
                    JavaSwitchArm {
                        pattern: JavaPattern::Default,
                        body: JavaBlock::new(vec![]),
                    },
                ],
            }]),
        ),
    ]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], declaration)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidStructure
            && value.message.contains("expression statement")
    }));
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::DuplicateDeclaration
            && value.message.contains("constant label")
    }));
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::TypeMismatch
            && value.message.contains("literal is not compatible")
    }));
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("dominated by an earlier pattern")
    }));
}

#[test]
fn valid_statement_expressions_and_switch_patterns_verify() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let string = JavaType::known(JavaKnownType::String);
    let assertion = JavaType::known(JavaKnownType::AssertionError);
    let constructor = JavaKnownConstructor::AssertionErrorString;
    let expression = JavaExpr {
        ty: assertion.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Known {
                constructor,
                owner: assertion,
                parameters: vec![string.clone()],
            },
            arguments: vec![JavaExpr::literal(
                string,
                JavaLiteral::String("discarded".to_owned()),
            )],
        },
    };
    let declaration = fixture_declaration(vec![structural_method(
        "validGrammar",
        JavaType::primitive(JavaPrimitive::Void),
        vec![parameter(int.clone(), "selector")],
        JavaBlock::new(vec![
            JavaStmt::Expression(expression),
            JavaStmt::Switch {
                value: JavaExpr::local(int, JavaIdentifier::from_portable("selector")),
                arms: vec![
                    JavaSwitchArm {
                        pattern: JavaPattern::Literal(JavaLiteral::I32(1)),
                        body: JavaBlock::new(vec![]),
                    },
                    JavaSwitchArm {
                        pattern: JavaPattern::Default,
                        body: JavaBlock::new(vec![]),
                    },
                ],
            },
        ]),
    )]);
    assert!(
        verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], declaration)],
        )
        .is_ok()
    );
}
