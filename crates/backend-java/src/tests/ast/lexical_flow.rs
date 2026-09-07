use super::{
    DiagnosticCode, JavaBlock, JavaDialect, JavaExpr, JavaIdentifier, JavaLexicalScope,
    JavaLiteral, JavaLocalFinality, JavaMethod, JavaMethodDeclaration, JavaModifier, JavaParameter,
    JavaPrimitive, JavaStmt, JavaType, block_guarantees_exit, fixture_declaration, parameter,
    structural_method, verify_block_scope, verify_fixture,
};

#[test]
fn lexical_verifier_rejects_unresolved_final_assignment_and_wrong_return() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let method = JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Static],
        type_parameters: vec![],
        return_type: boolean.clone(),
        name: JavaIdentifier::from_portable("invalid"),
        parameters: vec![JavaParameter {
            ty: int.clone(),
            name: JavaIdentifier::from_portable("fixed"),
            final_parameter: true,
        }],
        body: None,
    };
    let (mut scope, initial) = JavaLexicalScope::for_method(&method);
    assert!(initial.is_empty());
    let block = JavaBlock::new(vec![
        JavaStmt::Assign {
            target: JavaExpr::local(int.clone(), JavaIdentifier::from_portable("fixed")),
            value: JavaExpr::literal(int.clone(), JavaLiteral::I32(1)),
        },
        JavaStmt::Expression(JavaExpr::local(
            int.clone(),
            JavaIdentifier::from_portable("missing"),
        )),
        JavaStmt::Return(Some(JavaExpr::literal(int, JavaLiteral::I32(2)))),
    ]);
    let violations = verify_block_scope(&block, &mut scope, &boolean, false);
    assert!(
        violations
            .iter()
            .any(|value| value.code == DiagnosticCode::InvalidControlFlow)
    );
    assert!(
        violations
            .iter()
            .any(|value| value.code == DiagnosticCode::UnresolvedReference)
    );
    assert!(
        violations
            .iter()
            .any(|value| value.code == DiagnosticCode::TypeMismatch)
    );
    assert!(block_guarantees_exit(&block));
    assert!(!block_guarantees_exit(&JavaBlock::new(vec![])));
}

#[test]
fn lexical_verifier_proves_local_definite_assignment_across_branches() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let method = JavaMethod {
        declared: JavaMethodDeclaration::Structural,
        annotations: vec![],
        modifiers: vec![JavaModifier::Static],
        type_parameters: vec![],
        return_type: int.clone(),
        name: JavaIdentifier::from_portable("assignment"),
        parameters: vec![parameter(boolean.clone(), "condition")],
        body: None,
    };
    let local_name = JavaIdentifier::from_portable("result");
    let assignment = |value| JavaStmt::Assign {
        target: JavaExpr::local(int.clone(), local_name.clone()),
        value: JavaExpr::literal(int.clone(), JavaLiteral::I32(value)),
    };
    let declaration = || JavaStmt::Local {
        finality: JavaLocalFinality::Mutable,
        ty: int.clone(),
        name: local_name.clone(),
        value: None,
    };
    let result = || JavaStmt::Return(Some(JavaExpr::local(int.clone(), local_name.clone())));

    let (mut scope, initial) = JavaLexicalScope::for_method(&method);
    assert!(initial.is_empty());
    let unassigned = verify_block_scope(
        &JavaBlock::new(vec![declaration(), result()]),
        &mut scope,
        &int,
        false,
    );
    assert!(unassigned.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("before it is definitely assigned")
    }));

    let (mut scope, initial) = JavaLexicalScope::for_method(&method);
    assert!(initial.is_empty());
    let both_branches = verify_block_scope(
        &JavaBlock::new(vec![
            declaration(),
            JavaStmt::If {
                condition: JavaExpr::local(
                    boolean.clone(),
                    JavaIdentifier::from_portable("condition"),
                ),
                then_block: JavaBlock::new(vec![assignment(1)]),
                else_block: Some(JavaBlock::new(vec![assignment(2)])),
            },
            result(),
        ]),
        &mut scope,
        &int,
        false,
    );
    assert!(both_branches.is_empty(), "{both_branches:?}");

    let (mut scope, initial) = JavaLexicalScope::for_method(&method);
    assert!(initial.is_empty());
    let one_branch = verify_block_scope(
        &JavaBlock::new(vec![
            declaration(),
            JavaStmt::If {
                condition: JavaExpr::local(boolean, JavaIdentifier::from_portable("condition")),
                then_block: JavaBlock::new(vec![assignment(1)]),
                else_block: None,
            },
            result(),
        ]),
        &mut scope,
        &int,
        false,
    );
    assert!(one_branch.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("before it is definitely assigned")
    }));
}

#[test]
fn lexical_verifier_rejects_every_statement_after_noncompleting_control_flow() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let unreachable = fixture_declaration(vec![structural_method(
        "unreachable",
        int.clone(),
        vec![],
        JavaBlock::new(vec![
            JavaStmt::Return(Some(JavaExpr::literal(int.clone(), JavaLiteral::I32(1)))),
            JavaStmt::Return(Some(JavaExpr::literal(int, JavaLiteral::I32(2)))),
        ]),
    )]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], unreachable)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("unreachable Java statement")
    }));
}

#[test]
fn constant_loop_reachability_and_current_loop_breaks_are_structural() {
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let void = JavaType::primitive(JavaPrimitive::Void);

    let false_body = fixture_declaration(vec![structural_method(
        "falseBody",
        void,
        vec![],
        JavaBlock::new(vec![JavaStmt::While {
            condition: JavaExpr::literal(boolean.clone(), JavaLiteral::Boolean(false)),
            body: JavaBlock::new(vec![]),
        }]),
    )]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], false_body)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("while(false) body is unreachable")
    }));

    let after_infinite = fixture_declaration(vec![structural_method(
        "afterInfinite",
        int.clone(),
        vec![],
        JavaBlock::new(vec![
            JavaStmt::While {
                condition: JavaExpr::literal(boolean.clone(), JavaLiteral::Boolean(true)),
                body: JavaBlock::new(vec![]),
            },
            JavaStmt::Return(Some(JavaExpr::literal(int.clone(), JavaLiteral::I32(1)))),
        ]),
    )]);
    let diagnostics = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], after_infinite)],
    )
    .unwrap_err();
    assert!(diagnostics.iter().any(|value| {
        value.code == DiagnosticCode::InvalidControlFlow
            && value.message.contains("unreachable Java statement")
    }));

    let condition = JavaExpr::local(boolean.clone(), JavaIdentifier::from_portable("stop"));
    let valid = fixture_declaration(vec![
        structural_method(
            "infinite",
            int.clone(),
            vec![],
            JavaBlock::new(vec![JavaStmt::While {
                condition: JavaExpr::literal(boolean.clone(), JavaLiteral::Boolean(true)),
                body: JavaBlock::new(vec![]),
            }]),
        ),
        structural_method(
            "breaksCurrentLoop",
            int.clone(),
            vec![parameter(boolean.clone(), "stop")],
            JavaBlock::new(vec![
                JavaStmt::While {
                    condition: JavaExpr::literal(boolean.clone(), JavaLiteral::Boolean(true)),
                    body: JavaBlock::new(vec![JavaStmt::If {
                        condition,
                        then_block: JavaBlock::new(vec![JavaStmt::Break]),
                        else_block: None,
                    }]),
                },
                JavaStmt::Return(Some(JavaExpr::literal(int.clone(), JavaLiteral::I32(1)))),
            ]),
        ),
        structural_method(
            "nestedBreakDoesNotExitOuterLoop",
            int,
            vec![],
            JavaBlock::new(vec![JavaStmt::While {
                condition: JavaExpr::literal(boolean.clone(), JavaLiteral::Boolean(true)),
                body: JavaBlock::new(vec![JavaStmt::While {
                    condition: JavaExpr::literal(boolean, JavaLiteral::Boolean(true)),
                    body: JavaBlock::new(vec![JavaStmt::Break]),
                }]),
            }]),
        ),
    ]);
    let verification = verify_fixture(
        portable_codegen::TargetAstBuilder::new(JavaDialect),
        vec![(vec![], valid)],
    );
    assert!(verification.is_ok(), "{verification:?}");
}
