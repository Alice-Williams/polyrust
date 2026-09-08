//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaResultOperationsInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaResultOperationsInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaResultOperationsInput::IsOk { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::ValueResultIsOk, 1),
            result,
        ),
        JavaResultOperationsInput::IsErr { result, .. } => Plan::new(
            K::NegatedRuntimeCall(JavaRuntimeCallable::ValueResultIsOk, 1),
            result,
        ),
    })
}
