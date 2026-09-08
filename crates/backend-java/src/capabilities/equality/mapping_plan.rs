//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaEqualityInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaEqualityInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaEqualityInput::Equal { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::SemanticEqual, 2),
            result,
        ),
        JavaEqualityInput::NotEqual { result, .. } => Plan::new(
            K::NegatedRuntimeCall(JavaRuntimeCallable::SemanticEqual, 2),
            result,
        ),
    })
}
