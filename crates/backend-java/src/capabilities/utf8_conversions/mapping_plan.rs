//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaUtf8ConversionsInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaUtf8ConversionsInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaUtf8ConversionsInput::Encode { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::StringToUtf8, 1), result)
        }
        JavaUtf8ConversionsInput::DecodeChecked { result, .. } => Plan::new(
            K::FallibleRuntimeCall(JavaRuntimeCallable::StringFromUtf8, 1),
            result,
        ),
    })
}
