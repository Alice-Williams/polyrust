//! Java mapping for `OptionOperations`.

mod mapping_plan;

use portable_build::OptionOperations;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaType, JavaUnaryOperator},
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, conditional, runtime_call, unary},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaOptionOperationsInput {
    IsSome {
        operand: JavaExpr,
        result: JavaType,
    },
    IsNone {
        operand: JavaExpr,
        result: JavaType,
    },
    UnwrapOr {
        option: JavaExpr,
        fallback: Box<JavaExpr>,
        result: JavaType,
    },
}

fn lower_option_operations(
    input: JavaOptionOperationsInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(JavaIntrinsicExpr::Infallible(match input {
        JavaOptionOperationsInput::IsSome { operand, result } => {
            runtime_call(JavaRuntimeCallable::OptionIsSome, vec![operand], result)
        }
        JavaOptionOperationsInput::IsNone { operand, result } => {
            let some = runtime_call(
                JavaRuntimeCallable::OptionIsSome,
                vec![operand],
                result.clone(),
            );
            unary(JavaUnaryOperator::Not, some, result)
        }
        JavaOptionOperationsInput::UnwrapOr {
            option,
            fallback,
            result,
        } => {
            let present = runtime_call(
                JavaRuntimeCallable::OptionIsSome,
                vec![option.clone()],
                JavaType::primitive(crate::ast::JavaPrimitive::Boolean),
            );
            let value = runtime_call(
                JavaRuntimeCallable::OptionValue,
                vec![option],
                result.clone(),
            );
            conditional(present, value, *fallback, result)
        }
    }))
}

java_operation_mapping!(
    JavaOptionOperations,
    OptionOperations,
    JavaOptionOperationsInput,
    JavaIntrinsicExpr,
    lower_option_operations
);
