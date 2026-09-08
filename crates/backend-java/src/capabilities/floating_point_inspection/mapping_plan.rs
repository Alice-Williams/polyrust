//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaFloatingPointInspectionInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaKnownCallable;
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaFloatingPointInspectionInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaFloatingPointInspectionInput::Truncate { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::FloatTrunc, 1), result)
        }
        JavaFloatingPointInspectionInput::IsNan { result, .. } => {
            Plan::new(K::KnownCall(JavaKnownCallable::DoubleIsNaN, 1), result)
        }
        JavaFloatingPointInspectionInput::IsNegativeZero { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::FloatIsNegativeZero, 1),
            result,
        ),
        JavaFloatingPointInspectionInput::Absolute { result, .. } => {
            Plan::new(K::RuntimeCall(JavaRuntimeCallable::FloatAbs, 1), result)
        }
    })
}
