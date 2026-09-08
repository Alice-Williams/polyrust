use super::*;
#[test]
fn statement_and_control_plan_categories_reject_corruption() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let (plan, mut output) = checked(
        c::JavaLocalBindings,
        c::local_bindings::JavaLocalBindingsInput::Bind {
            name: "value".into(),
            ty: int.clone(),
            value: Box::new(i32_literal(1)),
        },
    );
    let c::local_bindings::JavaLocalBindingsNode::Statement(actual) = &mut output else {
        panic!()
    };
    **actual = JavaStmt::Return(None);
    assert!(!plan.verify_output(&output));

    let (plan, mut output) = checked(
        c::JavaLoops,
        c::loops::JavaLoopsInput::ForEach {
            binding_type: int.clone(),
            binding: "item".into(),
            iterable: Box::new(JavaExpr::local(
                JavaType::generic(JavaKnownType::List, vec![int.clone().boxed()]),
                identifier("items"),
            )),
            body: JavaBlock::new(vec![]),
        },
    );
    let c::loops::JavaLoopsNode::Statement(actual) = &mut output else {
        panic!()
    };
    **actual = JavaStmt::Return(None);
    assert!(!plan.verify_output(&output));

    let (plan, mut output) = checked(
        c::JavaConditionals,
        c::conditionals::JavaConditionalsInput::Value(Box::new(
            c::conditionals::JavaConditionalValueInput {
                prefix: vec![],
                condition: bool_literal(true),
                result_name: identifier("result"),
                result_type: int.clone(),
                then_block: JavaBlock::new(vec![]),
                else_block: JavaBlock::new(vec![]),
            },
        )),
    );
    let c::conditionals::JavaConditionalsNode::Value { statements, .. } = &mut output;
    statements.pop();
    assert!(!plan.verify_output(&output));

    let (plan, mut output) = checked(
        c::JavaPatternMatching,
        c::pattern_matching::JavaPatternMatchingInput::Pattern(Box::new(
            c::pattern_matching::JavaPatternInput::Wildcard,
        )),
    );
    let c::pattern_matching::JavaPatternMatchingNode::Pattern(actual) = &mut output else {
        panic!()
    };
    actual.condition = bool_literal(false);
    assert!(!plan.verify_output(&output));
    let (plan, mut output) = checked(
        c::JavaPatternMatching,
        c::pattern_matching::JavaPatternMatchingInput::Match(Box::new(
            c::pattern_matching::JavaMatchInput {
                prefix: vec![],
                matched: i32_literal(1),
                matched_name: identifier("matched"),
                result_name: identifier("result"),
                result_type: int,
                arms: vec![],
            },
        )),
    );
    let c::pattern_matching::JavaPatternMatchingNode::Match(actual) = &mut output else {
        panic!()
    };
    actual.statements.pop();
    assert!(!plan.verify_output(&output));
}

#[test]
fn result_propagation_and_test_case_owned_helpers_are_checked() {
    use crate::dialect::JavaRuntimeCallable;
    let int = JavaType::primitive(JavaPrimitive::Int);
    let call = crate::lower::runtime_call(
        JavaRuntimeCallable::Ok,
        vec![i32_literal(1)],
        JavaType::generic(JavaKnownType::RuntimeResult, vec![int.clone().boxed()]),
    );
    let (plan, mut output) = checked(
        c::JavaResultPropagation,
        c::result_propagation::JavaResultPropagationInput {
            prefix: vec![],
            call: call.clone(),
            result_name: identifier("result"),
            value_type: int,
            callable_result_type: call.ty.clone(),
        },
    );
    output.value = i32_literal(1);
    assert!(!plan.verify_output(&output));
    for expected in [
        c::portable_tests::JavaPortableTestExpectation::Value(i32_literal(1)),
        c::portable_tests::JavaPortableTestExpectation::Error(crate::lower::string_literal(
            "overflow",
        )),
    ] {
        let (plan, mut output) = checked(
            c::JavaPortableTests,
            c::portable_tests::JavaPortableTestsInput::Case(Box::new(
                c::portable_tests::JavaPortableTestCaseInput {
                    index: 0,
                    name: "case".into(),
                    actual: call.clone(),
                    expected,
                },
            )),
        );
        let c::portable_tests::JavaPortableTestsNode::Case(actual) = &mut output else {
            panic!()
        };
        actual.pop();
        assert!(!plan.verify_output(&output));
    }
}
