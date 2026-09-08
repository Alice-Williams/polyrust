//! Java mapping for `FloatingPointInspection`.

mod mapping_plan;

use portable_build::FloatingPointInspection;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaType},
    dialect::{JavaKnownCallable, JavaRuntimeCallable},
    lower::{JavaIntrinsicExpr, known_call, runtime_call},
};

#[doc(hidden)]
#[derive(Clone)]
pub enum JavaFloatingPointInspectionInput {
    Truncate { operand: JavaExpr, result: JavaType },
    IsNan { operand: JavaExpr, result: JavaType },
    IsNegativeZero { operand: JavaExpr, result: JavaType },
    Absolute { operand: JavaExpr, result: JavaType },
}

fn lower_floating_point_inspection(
    input: JavaFloatingPointInspectionInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    Ok(JavaIntrinsicExpr::Infallible(match input {
        JavaFloatingPointInspectionInput::Truncate { operand, result } => {
            runtime_call(JavaRuntimeCallable::FloatTrunc, vec![operand], result)
        }
        JavaFloatingPointInspectionInput::IsNan { operand, .. } => {
            known_call(JavaKnownCallable::DoubleIsNaN, vec![operand])
        }
        JavaFloatingPointInspectionInput::IsNegativeZero { operand, result } => runtime_call(
            JavaRuntimeCallable::FloatIsNegativeZero,
            vec![operand],
            result,
        ),
        JavaFloatingPointInspectionInput::Absolute { operand, result } => {
            runtime_call(JavaRuntimeCallable::FloatAbs, vec![operand], result)
        }
    }))
}

java_operation_mapping!(
    JavaFloatingPointInspection,
    FloatingPointInspection,
    JavaFloatingPointInspectionInput,
    JavaIntrinsicExpr,
    lower_floating_point_inspection
);
