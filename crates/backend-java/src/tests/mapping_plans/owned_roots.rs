use super::*;
use crate::{
    ast::*,
    capabilities as c,
    lower::{i32_literal, runtime_call, string_literal},
};
use crate::{dialect::JavaRuntimeCallable, lower::JavaIntrinsicExpr};
use c::support::plans::JavaRepresentation as R;

#[test]
fn runtime_negation_authenticates_its_owned_helper() {
    let input = c::equality::JavaEqualityInput::NotEqual {
        left: i32_literal(1),
        right: i32_literal(2),
        result: JavaType::primitive(JavaPrimitive::Boolean),
    };
    let (plan, mut output) = checked(c::JavaEquality, input);
    let JavaIntrinsicExpr::Infallible(value) = &mut output else {
        panic!()
    };
    let JavaExprKind::Unary { operand, .. } = &mut value.kind else {
        panic!()
    };
    let JavaExprKind::Call {
        callable: JavaCallableRef::Runtime { callable, .. },
        ..
    } = &mut operand.kind
    else {
        panic!()
    };
    *callable = JavaRuntimeCallable::DeepEqual;
    assert!(
        !plan.verify_output(&output),
        "same-shaped but wrong runtime helper"
    );
}
#[test]
fn conditional_owned_helpers_and_fallback_holes_are_distinct() {
    let int = JavaType::primitive(JavaPrimitive::Int);
    let option = JavaType::generic(JavaKnownType::RuntimeOption, vec![int.clone().boxed()]);
    let input = c::option_operations::JavaOptionOperationsInput::UnwrapOr {
        option: runtime_call(JavaRuntimeCallable::OptionNone, vec![], option),
        fallback: Box::new(i32_literal(1)),
        result: int,
    };
    let (plan, mut output) = checked(c::JavaOptionOperations, input);
    let JavaIntrinsicExpr::Infallible(value) = &mut output else {
        panic!()
    };
    let JavaExprKind::Conditional { when_false, .. } = &mut value.kind else {
        panic!()
    };
    **when_false = i32_literal(3);
    assert!(plan.verify_output(&output), "fallback is an input hole");
    let JavaIntrinsicExpr::Infallible(value) = &mut output else {
        panic!()
    };
    let JavaExprKind::Conditional { when_true, .. } = &mut value.kind else {
        panic!()
    };
    **when_true = i32_literal(3);
    assert!(
        !plan.verify_output(&output),
        "option payload getter is mapping-owned"
    );
}
#[test]
fn nested_string_helpers_are_authenticated() {
    let (plan, mut output) = checked(
        c::JavaStringTransformation,
        c::string_transformation::JavaStringTransformationInput::StripPrefix {
            source: string_literal("prefix-text"),
            prefix: string_literal("prefix-"),
            result: JavaType::known(JavaKnownType::String),
        },
    );
    let JavaIntrinsicExpr::Infallible(value) = &mut output else {
        panic!()
    };
    let JavaExprKind::Conditional { when_true, .. } = &mut value.kind else {
        panic!()
    };
    let JavaExprKind::Call { arguments, .. } = &mut when_true.kind else {
        panic!()
    };
    arguments[0] = i32_literal(0);
    assert!(
        !plan.verify_output(&output),
        "substring offset must be the owned prefix-length call"
    );
}
#[test]
fn enum_type_plans_retain_native_and_payload_representation() {
    let (enumeration, _) = super::declarations::symbols();
    for (shape, expected) in [
        (c::enums::JavaEnumShape::Native, R::Direct),
        (c::enums::JavaEnumShape::Payload, R::TaggedValue),
    ] {
        let (plan, _) = checked(
            c::JavaEnums,
            c::enums::JavaEnumsInput::Type { enumeration, shape },
        );
        assert_eq!(plan.representation(), expected);
    }
}
