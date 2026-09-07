use super::*;

#[test]
fn portable_error_expectation_compares_code_not_diagnostic_message() {
    let expected = string_literal("overflow");
    let actual = runtime_call(
        JavaRuntimeCallable::Fail,
        vec![
            expected.clone(),
            string_literal("the diagnostic message may differ"),
        ],
        JavaType::generic(
            JavaKnownType::RuntimeResult,
            vec![JavaType::known(JavaKnownType::RuntimeUnit)],
        ),
    );
    let JavaPortableTestsNode::Case(statements) = JavaPortableTests
        .lower(
            &mut (),
            JavaPortableTestsInput::Case(Box::new(JavaPortableTestCaseInput {
                index: 0,
                name: "code_contract".to_owned(),
                actual,
                expected: JavaPortableTestExpectation::Error(expected.clone()),
            })),
        )
        .expect("error expectation maps")
    else {
        panic!("expected case statements")
    };
    assert_eq!(
        statements.len(),
        3,
        "capture result, assert failure, compare portable code"
    );
    let JavaStmt::If { condition, .. } = &statements[2] else {
        panic!("comparison assertion")
    };
    let JavaExprKind::Unary {
        operator: JavaUnaryOperator::Not,
        operand,
    } = &condition.kind
    else {
        panic!("assert true")
    };
    let JavaExprKind::Call {
        callable:
            JavaCallableRef::Runtime {
                callable: JavaRuntimeCallable::SemanticEqual,
                ..
            },
        arguments,
        ..
    } = &operand.kind
    else {
        panic!("portable equality")
    };
    assert_eq!(arguments[1], expected);
    assert!(matches!(
        &arguments[0].kind,
        JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                origin: JavaMemberOrigin::Runtime(JavaRuntimeMember::ErrorCode),
                ..
            },
            ..
        }
    ));
}
