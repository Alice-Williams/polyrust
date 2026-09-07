//! Java mapping for `IntegerBitwise`.

use portable_build::IntegerBitwise;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaBinaryOperator, JavaExpr, JavaType, JavaUnaryOperator},
    lower::{JavaIntrinsicExpr, binary, unary},
};

#[doc(hidden)]
pub enum JavaIntegerBitwiseInput {
    Not {
        operand: JavaExpr,
        result: JavaType,
    },
    And {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    Or {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
    Xor {
        left: JavaExpr,
        right: JavaExpr,
        result: JavaType,
    },
}

fn lower_integer_bitwise(
    input: JavaIntegerBitwiseInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(JavaIntrinsicExpr::Direct(match input {
        JavaIntegerBitwiseInput::Not { operand, result } => {
            unary(JavaUnaryOperator::BitNot, operand, result)
        }
        JavaIntegerBitwiseInput::And {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::BitAnd, left, right, result),
        JavaIntegerBitwiseInput::Or {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::BitOr, left, right, result),
        JavaIntegerBitwiseInput::Xor {
            left,
            right,
            result,
        } => binary(JavaBinaryOperator::BitXor, left, right, result),
    }))
}

java_operation_mapping!(
    JavaIntegerBitwise,
    IntegerBitwise,
    JavaIntegerBitwiseInput,
    JavaIntrinsicExpr,
    lower_integer_bitwise
);
