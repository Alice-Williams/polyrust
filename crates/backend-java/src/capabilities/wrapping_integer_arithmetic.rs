//! Java mapping for `WrappingIntegerArithmetic`.

use portable_build::WrappingIntegerArithmetic;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaBinaryOperator, JavaExpr, JavaType, JavaUnaryOperator},
    lower::{JavaIntrinsicExpr, binary, unary},
};

#[doc(hidden)]
pub enum JavaWrappingIntegerArithmeticInput {
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
}

fn lower_wrapping_integer_arithmetic(
    input: JavaWrappingIntegerArithmeticInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(JavaIntrinsicExpr::Direct(match input {
        JavaWrappingIntegerArithmeticInput::Neg { operand, result } => {
            unary(JavaUnaryOperator::Negate, operand, result)
        }
        JavaWrappingIntegerArithmeticInput::Add {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::Add, left, right, result),
        JavaWrappingIntegerArithmeticInput::Subtract {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::Subtract, left, right, result),
        JavaWrappingIntegerArithmeticInput::Multiply {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::Multiply, left, right, result),
    }))
}

java_operation_mapping!(
    JavaWrappingIntegerArithmetic,
    WrappingIntegerArithmetic,
    JavaWrappingIntegerArithmeticInput,
    JavaIntrinsicExpr,
    lower_wrapping_integer_arithmetic
);
