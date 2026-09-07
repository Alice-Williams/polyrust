//! Java mapping for `Ordering`.

use portable_build::Ordering;

use super::support::java_operation_mapping;
use crate::{
    ast::{
        JavaBinaryOperator, JavaExpr, JavaKnownType, JavaMemberOrigin, JavaPrimitive,
        JavaRuntimeMember, JavaType,
    },
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, binary, i32_literal, member_call, runtime_call},
};

#[doc(hidden)]
pub enum JavaOrderingInput {
    Less {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    LessEqual {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    Greater {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    GreaterEqual {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
}

fn lower_ordering(
    input: JavaOrderingInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    let (operator, left, right, result) = match input {
        JavaOrderingInput::Less {
            left,
            right,
            result,
        } => (JavaBinaryOperator::Less, left, right, result),
        JavaOrderingInput::LessEqual {
            left,
            right,
            result,
        } => (JavaBinaryOperator::LessEqual, left, right, result),
        JavaOrderingInput::Greater {
            left,
            right,
            result,
        } => (JavaBinaryOperator::Greater, left, right, result),
        JavaOrderingInput::GreaterEqual {
            left,
            right,
            result,
        } => (JavaBinaryOperator::GreaterEqual, left, right, result),
    };
    Ok(JavaIntrinsicExpr::Direct(
        if left.ty == JavaType::known(JavaKnownType::String) {
            binary(
                operator,
                runtime_call(
                    JavaRuntimeCallable::CompareScalarStrings,
                    vec![left, right],
                    JavaType::primitive(JavaPrimitive::Int),
                ),
                i32_literal(0),
                result,
            )
        } else if left.ty == JavaType::known(JavaKnownType::RuntimeScalar) {
            binary(
                operator,
                member_call(
                    left,
                    "value",
                    vec![],
                    JavaType::primitive(JavaPrimitive::Int),
                    JavaMemberOrigin::Runtime(JavaRuntimeMember::ScalarValue),
                ),
                member_call(
                    right,
                    "value",
                    vec![],
                    JavaType::primitive(JavaPrimitive::Int),
                    JavaMemberOrigin::Runtime(JavaRuntimeMember::ScalarValue),
                ),
                result,
            )
        } else {
            binary(operator, left, right, result)
        },
    ))
}

java_operation_mapping!(
    JavaOrdering,
    Ordering,
    JavaOrderingInput,
    JavaIntrinsicExpr,
    lower_ordering
);
