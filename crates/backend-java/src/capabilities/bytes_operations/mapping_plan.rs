//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaBytesOperationsInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaBytesOperationsInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaBytesOperationsInput::Length { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::BytesLength, 1), result)
        }
        JavaBytesOperationsInput::IsEmpty { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::BytesIsEmpty, 1), result)
        }
        JavaBytesOperationsInput::Concat { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::BytesConcat, 2), result)
        }
        JavaBytesOperationsInput::ReplaceAll { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::BytesReplaceAll, 3),
            result,
        ),
    })
}
