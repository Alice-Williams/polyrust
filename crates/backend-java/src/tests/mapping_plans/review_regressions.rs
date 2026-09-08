//! Independent mutations for the broad review findings.
use super::*;
use crate::dialect::JavaRuntimeCallable;
use crate::{
    ast::*,
    capabilities as c,
    lower::{bool_literal, i32_literal, identifier, runtime_call, string_literal},
};

#[test]
fn ordering_rejects_non_orderable_java_representations() {
    let (record, _) = super::declarations::symbols();
    for ty in [
        JavaType::primitive(JavaPrimitive::Boolean),
        JavaType::primitive(JavaPrimitive::Void),
        JavaType::Boxed(JavaPrimitive::Int),
        JavaType::known(JavaKnownType::RuntimeBytes),
        JavaType::Reference(JavaTypeName::Generated(record)),
        JavaType::generic(
            JavaKnownType::List,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        ),
        JavaType::generic(
            JavaKnownType::RuntimeOption,
            vec![JavaType::Boxed(JavaPrimitive::Int)],
        ),
    ] {
        let input = c::ordering::JavaOrderingInput::Less {
            left: JavaExpr::local(ty.clone(), identifier("left")),
            right: JavaExpr::local(ty, identifier("right")),
            result: JavaType::primitive(JavaPrimitive::Boolean),
        };
        assert!(c::JavaOrdering.select_plan(&input).is_err());
        assert!(c::JavaOrdering.lower(&mut (), input.clone()).is_err());
        assert!(
            c::java_capabilities()
                .mapping_for::<portable_build::Ordering>()
                .lower(&mut (), input)
                .is_err()
        );
    }
}

#[test]
fn structured_boolean_owned_nodes_authenticate_types() {
    use c::boolean_logic::{JavaBooleanLogicInput as I, JavaBooleanLogicPlan as Operand};
    for corrupt_condition in [false, true] {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let (plan, mut output) = checked(
            c::JavaBooleanLogic,
            I::And {
                left: Operand {
                    statements: vec![],
                    value: bool_literal(true),
                },
                right: Operand {
                    statements: vec![JavaStmt::Local {
                        finality: JavaLocalFinality::Final,
                        ty: boolean.clone(),
                        name: identifier("right_prefix"),
                        value: Some(bool_literal(true)),
                    }],
                    value: bool_literal(false),
                },
                result_name: identifier("result"),
                result: boolean,
            },
        );
        let JavaStmt::If {
            condition,
            then_block,
            ..
        } = &mut output.statements[1]
        else {
            panic!()
        };
        if corrupt_condition {
            condition.ty = JavaType::primitive(JavaPrimitive::Int);
        } else {
            let JavaStmt::Assign { value, .. } = &mut then_block.statements[0] else {
                panic!()
            };
            value.ty = JavaType::primitive(JavaPrimitive::Int);
        }
        assert!(!plan.verify_output(&output));
    }
}
#[derive(Clone, Copy)]
enum CaseFault {
    Status,
    Comparison,
    Payload,
    Error,
    Code,
    Message,
}
fn case(error: bool) -> c::portable_tests::JavaPortableTestsInput {
    use c::portable_tests::*;
    let int = JavaType::primitive(JavaPrimitive::Int);
    JavaPortableTestsInput::Case(Box::new(JavaPortableTestCaseInput {
        index: 0,
        name: "owned_types".into(),
        actual: runtime_call(
            JavaRuntimeCallable::Ok,
            vec![i32_literal(1)],
            JavaType::generic(JavaKnownType::RuntimeResult, vec![int.boxed()]),
        ),
        expected: if error {
            JavaPortableTestExpectation::Error(string_literal("failure"))
        } else {
            JavaPortableTestExpectation::Value(i32_literal(1))
        },
    }))
}
fn corrupt_assertion_message(statement: &mut JavaStmt) {
    let JavaStmt::If { then_block, .. } = statement else {
        panic!()
    };
    let JavaStmt::ThrowAssertion(message) = &mut then_block.statements[0] else {
        panic!()
    };
    *message = string_literal("different but still well-typed message");
}
#[test]
fn portable_test_owned_messages_reject_content_only_mutations() {
    for error in [false, true] {
        for assertion in [1, 2] {
            let (plan, mut output) = checked(c::JavaPortableTests, case(error));
            let c::portable_tests::JavaPortableTestsNode::Case(statements) = &mut output else {
                panic!()
            };
            corrupt_assertion_message(&mut statements[assertion]);
            assert!(!plan.verify_output(&output));
        }
    }
    let (plan, mut output) = checked(
        c::JavaPortableTests,
        c::portable_tests::JavaPortableTestsInput::Harness(
            c::portable_tests::JavaPortableTestHarnessInput {
                class_name: "GeneratedTest".into(),
                cases: vec![],
                expected_test_count: 0,
            },
        ),
    );
    let c::portable_tests::JavaPortableTestsNode::Harness(declaration) = &mut output else {
        panic!()
    };
    let JavaMember::Method(main) = &mut declaration.members[1] else {
        panic!()
    };
    corrupt_assertion_message(main.body.as_mut().unwrap().statements.last_mut().unwrap());
    assert!(!plan.verify_output(&output));
}
fn asserted_mut(statement: &mut JavaStmt, direct: bool) -> &mut JavaExpr {
    let JavaStmt::If { condition, .. } = statement else {
        panic!()
    };
    if direct {
        condition
    } else {
        let JavaExprKind::Unary { operand, .. } = &mut condition.kind else {
            panic!()
        };
        operand
    }
}
#[test]
fn portable_test_owned_helper_types_reject_independent_mutations() {
    for (error, fault) in [
        (false, CaseFault::Status),
        (true, CaseFault::Status),
        (false, CaseFault::Comparison),
        (true, CaseFault::Comparison),
        (false, CaseFault::Payload),
        (true, CaseFault::Error),
        (true, CaseFault::Code),
        (false, CaseFault::Message),
    ] {
        let (plan, mut output) = checked(c::JavaPortableTests, case(error));
        let c::portable_tests::JavaPortableTestsNode::Case(statements) = &mut output else {
            panic!()
        };
        let corrupted = match fault {
            CaseFault::Status => asserted_mut(&mut statements[1], error),
            CaseFault::Comparison => asserted_mut(&mut statements[2], false),
            CaseFault::Payload | CaseFault::Code | CaseFault::Error => {
                let comparison = asserted_mut(&mut statements[2], false);
                let JavaExprKind::Call { arguments, .. } = &mut comparison.kind else {
                    panic!()
                };
                if matches!(fault, CaseFault::Error) {
                    let JavaExprKind::Call {
                        receiver: Some(error),
                        ..
                    } = &mut arguments[0].kind
                    else {
                        panic!()
                    };
                    error
                } else {
                    &mut arguments[0]
                }
            }
            CaseFault::Message => {
                let JavaStmt::If { then_block, .. } = &mut statements[1] else {
                    panic!()
                };
                let JavaStmt::ThrowAssertion(message) = &mut then_block.statements[0] else {
                    panic!()
                };
                message
            }
        };
        corrupted.ty = JavaType::primitive(JavaPrimitive::Long);
        assert!(!plan.verify_output(&output));
    }
}
