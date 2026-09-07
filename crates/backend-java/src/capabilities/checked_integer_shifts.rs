//! Java mapping for `CheckedIntegerShifts`.

use portable_build::CheckedIntegerShifts;

use super::support::java_operation_mapping;
use crate::{
    ast::{JavaExpr, JavaPrimitive, JavaType},
    dialect::JavaRuntimeCallable,
    lower::{JavaIntrinsicExpr, diagnostic, runtime_fallible},
};

#[doc(hidden)]
pub enum JavaCheckedIntegerShiftsInput {
    Left {
        value: JavaExpr,
        distance: JavaExpr,
        result: JavaType,
    },
    Right {
        value: JavaExpr,
        distance: JavaExpr,
        result: JavaType,
    },
}

fn lower_checked_integer_shifts(
    input: JavaCheckedIntegerShiftsInput,
) -> Result<JavaIntrinsicExpr, Vec<portable_diagnostics::Diagnostic>> {
    let (left_shift, value, distance, result) = match input {
        JavaCheckedIntegerShiftsInput::Left {
            value,
            distance,
            result,
        } => (true, value, distance, result),
        JavaCheckedIntegerShiftsInput::Right {
            value,
            distance,
            result,
        } => (false, value, distance, result),
    };
    let wide = match value.ty {
        JavaType::Primitive(JavaPrimitive::Int) => false,
        JavaType::Primitive(JavaPrimitive::Long) => true,
        _ => {
            return Err(vec![diagnostic(
                "checked shift requires a Java int or long",
            )]);
        }
    };
    let callable = match (left_shift, wide) {
        (true, false) => JavaRuntimeCallable::CheckedShiftLeftI32,
        (true, true) => JavaRuntimeCallable::CheckedShiftLeftI64,
        (false, false) => JavaRuntimeCallable::CheckedShiftRightI32,
        (false, true) => JavaRuntimeCallable::CheckedShiftRightI64,
    };
    Ok(runtime_fallible(callable, vec![value, distance], result))
}

java_operation_mapping!(
    JavaCheckedIntegerShifts,
    CheckedIntegerShifts,
    JavaCheckedIntegerShiftsInput,
    JavaIntrinsicExpr,
    lower_checked_integer_shifts
);
