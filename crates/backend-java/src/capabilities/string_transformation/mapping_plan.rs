//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaStringTransformationInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaStringTransformationInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaStringTransformationInput::StripPrefix { result, .. } => {
            Plan::new(K::StripPrefix, result)
        }
        JavaStringTransformationInput::TruncateUtf8Bytes { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::StringTruncateUtf8Bytes, 2),
            result,
        ),
        JavaStringTransformationInput::TrimStart { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::StringTrimStart, 2),
            result,
        ),
        JavaStringTransformationInput::TrimEnd { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::StringTrimEnd, 2),
            result,
        ),
        JavaStringTransformationInput::SliceScalars { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::StringSliceScalars, 3),
            result,
        ),
        JavaStringTransformationInput::ReplaceAll { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::StringReplaceAll, 3),
            result,
        ),
        JavaStringTransformationInput::ReplaceMany {
            result,
            replacements,
            ..
        } => Plan::new(K::ReplaceMany(replacements.len()), result),
    })
}
