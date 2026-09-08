//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaListOperationsInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaListOperationsInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaListOperationsInput::Length { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::ListLength, 1), result)
        }
        JavaListOperationsInput::IsEmpty { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::ListIsEmpty, 1), result)
        }
        JavaListOperationsInput::GetChecked { result, .. } => Plan::new(
            K::FallibleRuntimeCall(JavaRuntimeCallable::ListGet, 2),
            result,
        ),
        JavaListOperationsInput::Append { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::ListAppend, 2), result)
        }
        JavaListOperationsInput::Concat { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::ListConcat, 2), result)
        }
        JavaListOperationsInput::Contains { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::ListContains, 2), result)
        }
        JavaListOperationsInput::IndexOf { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::ListIndexOf, 2), result)
        }
    })
}
