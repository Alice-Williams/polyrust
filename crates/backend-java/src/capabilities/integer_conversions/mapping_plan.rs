//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaIntegerConversionsInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaIntegerConversionsInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaIntegerConversionsInput::WidenI32ToI64 { result, .. } => Plan::new(K::Cast, result),
        JavaIntegerConversionsInput::NarrowI64ToI32Checked { result, .. } => Plan::new(
            K::FallibleRuntimeCall(JavaRuntimeCallable::NarrowI64ToI32, 1),
            result,
        ),
    })
}
