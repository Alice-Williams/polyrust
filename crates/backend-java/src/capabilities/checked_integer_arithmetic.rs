//! Java mapping for `CheckedIntegerArithmetic`.

use portable_build::CheckedIntegerArithmetic;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaPrimitive, JavaType},
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, diagnostic, runtime_fallible},
};

#[doc(hidden)]
pub enum JavaCheckedIntegerArithmeticInput {
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

fn lower_checked_integer_arithmetic(
    input: JavaCheckedIntegerArithmeticInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    enum Operation {
        Neg,
        Add,
        Subtract,
        Multiply,
        Divide,
        Remainder,
    }

    let (operation, operands, result) = match input {
        JavaCheckedIntegerArithmeticInput::Neg { operand, result } => {
            (Operation::Neg, vec![operand], result)
        }
        JavaCheckedIntegerArithmeticInput::Add {
            left,
            right,
            result,
        } => (Operation::Add, vec![left, right], result),
        JavaCheckedIntegerArithmeticInput::Subtract {
            left,
            right,
            result,
        } => (Operation::Subtract, vec![left, right], result),
        JavaCheckedIntegerArithmeticInput::Multiply {
            left,
            right,
            result,
        } => (Operation::Multiply, vec![left, right], result),
        JavaCheckedIntegerArithmeticInput::Divide {
            left,
            right,
            result,
        } => (Operation::Divide, vec![left, right], result),
        JavaCheckedIntegerArithmeticInput::Remainder {
            left,
            right,
            result,
        } => (Operation::Remainder, vec![left, right], result),
    };
    let wide = match operands[0].ty {
        JavaType::Primitive(JavaPrimitive::Int) => false,
        JavaType::Primitive(JavaPrimitive::Long) => true,
        _ => {
            return Err(vec![diagnostic(
                "checked arithmetic requires a Java int or long",
            )]);
        }
    };
    let callable = match (operation, wide) {
        (Operation::Neg, false) => JavaRuntimeCallable::CheckedNegI32,
        (Operation::Neg, true) => JavaRuntimeCallable::CheckedNegI64,
        (Operation::Add, false) => JavaRuntimeCallable::CheckedAddI32,
        (Operation::Add, true) => JavaRuntimeCallable::CheckedAddI64,
        (Operation::Subtract, false) => JavaRuntimeCallable::CheckedSubI32,
        (Operation::Subtract, true) => JavaRuntimeCallable::CheckedSubI64,
        (Operation::Multiply, false) => JavaRuntimeCallable::CheckedMulI32,
        (Operation::Multiply, true) => JavaRuntimeCallable::CheckedMulI64,
        (Operation::Divide, false) => JavaRuntimeCallable::CheckedDivI32,
        (Operation::Divide, true) => JavaRuntimeCallable::CheckedDivI64,
        (Operation::Remainder, false) => JavaRuntimeCallable::CheckedRemI32,
        (Operation::Remainder, true) => JavaRuntimeCallable::CheckedRemI64,
    };
    Ok(runtime_fallible(callable, operands, result))
}

java_operation_mapping!(
    JavaCheckedIntegerArithmetic,
    CheckedIntegerArithmetic,
    JavaCheckedIntegerArithmeticInput,
    JavaIntrinsicExpr,
    lower_checked_integer_arithmetic
);
