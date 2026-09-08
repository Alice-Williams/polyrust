//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaCheckedIntegerShiftsInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaCheckedIntegerShiftsInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaCheckedIntegerShiftsInput::Left { value, result, .. } => {
            let callable = match value.ty {
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Int) => {
                    JavaRuntimeCallable::CheckedShiftLeftI32
                }
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Long) => {
                    JavaRuntimeCallable::CheckedShiftLeftI64
                }
                _ => {
                    return Err(vec![crate::lower::diagnostic(
                        "checked integer plan requires int or long",
                    )]);
                }
            };
            Plan::new(K::FallibleRuntimeCall(callable, 2), result)
        }
        JavaCheckedIntegerShiftsInput::Right { value, result, .. } => {
            let callable = match value.ty {
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Int) => {
                    JavaRuntimeCallable::CheckedShiftRightI32
                }
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Long) => {
                    JavaRuntimeCallable::CheckedShiftRightI64
                }
                _ => {
                    return Err(vec![crate::lower::diagnostic(
                        "checked integer plan requires int or long",
                    )]);
                }
            };
            Plan::new(K::FallibleRuntimeCall(callable, 2), result)
        }
    })
}
