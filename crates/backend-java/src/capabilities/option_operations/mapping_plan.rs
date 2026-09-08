//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaOptionOperationsInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaOptionOperationsInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaOptionOperationsInput::IsSome { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::OptionIsSome, 1), result)
        }
        JavaOptionOperationsInput::IsNone { result, .. } => Plan::new(
            K::NegatedRuntimeCall(JavaRuntimeCallable::OptionIsSome, 1),
            result,
        ),
        JavaOptionOperationsInput::UnwrapOr { result, .. } => Plan::new(K::OptionUnwrapOr, result),
    })
}
