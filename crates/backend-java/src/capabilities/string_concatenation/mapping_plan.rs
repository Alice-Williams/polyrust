//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaStringConcatenationInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaKnownMethod;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaStringConcatenationInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(Plan::new(
        K::KnownMember(JavaKnownMethod::StringConcat, 1),
        &input.result,
    ))
}
