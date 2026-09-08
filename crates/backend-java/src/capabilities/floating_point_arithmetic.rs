//! Java mapping for `FloatingPointArithmetic`.

mod mapping_plan;

use portable_build::FloatingPointArithmetic;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaBinaryOperator, JavaExpr, JavaType, JavaUnaryOperator},
    lower::{JavaIntrinsicExpr, binary, unary},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaFloatingPointArithmeticInput {
    Neg {
        operand: JavaExpr,
        result: JavaType,
    },
    Add {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    Subtract {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    Multiply {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    Divide {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    Remainder {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
}

fn lower_floating_point_arithmetic(
    input: JavaFloatingPointArithmeticInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(JavaIntrinsicExpr::Infallible(match input {
        JavaFloatingPointArithmeticInput::Neg { operand, result } => {
            unary(JavaUnaryOperator::Negate, operand, result)
        }
        JavaFloatingPointArithmeticInput::Add {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::Add, left, right, result),
        JavaFloatingPointArithmeticInput::Subtract {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::Subtract, left, right, result),
        JavaFloatingPointArithmeticInput::Multiply {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::Multiply, left, right, result),
        JavaFloatingPointArithmeticInput::Divide {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::Divide, left, right, result),
        JavaFloatingPointArithmeticInput::Remainder {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::Remainder, left, right, result),
    }))
}

java_operation_mapping!(
    JavaFloatingPointArithmetic,
    FloatingPointArithmetic,
    JavaFloatingPointArithmeticInput,
    JavaIntrinsicExpr,
    lower_floating_point_arithmetic
);
