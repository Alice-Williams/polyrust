//! Java mapping for `ResultOperations`.

mod mapping_plan;

use portable_build::ResultOperations;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaType, JavaUnaryOperator},
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, runtime_call, unary},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaResultOperationsInput {
    IsOk { operand: JavaExpr, result: JavaType },
    IsErr { operand: JavaExpr, result: JavaType },
}

fn lower_result_operations(
    input: JavaResultOperationsInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(JavaIntrinsicExpr::Infallible(match input {
        JavaResultOperationsInput::IsOk { operand, result } => {
            runtime_call(JavaRuntimeCallable::ValueResultIsOk, vec![operand], result)
        }
        JavaResultOperationsInput::IsErr { operand, result } => {
            let ok = runtime_call(
                JavaRuntimeCallable::ValueResultIsOk,
                vec![operand],
                result.clone(),
            );
            unary(JavaUnaryOperator::Not, ok, result)
        }
    }))
}

java_operation_mapping!(
    JavaResultOperations,
    ResultOperations,
    JavaResultOperationsInput,
    JavaIntrinsicExpr,
    lower_result_operations
);
