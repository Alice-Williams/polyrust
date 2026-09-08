//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaStringInspectionInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaKnownMethod;
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaStringInspectionInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaStringInspectionInput::ScalarLength { result, .. } => Plan::new(
            K::FallibleRuntimeCall(JavaRuntimeCallable::ScalarLength, 1),
            result,
        ),
        JavaStringInspectionInput::Utf16Length { result, .. } => Plan::new(K::Utf16Length, result),
        JavaStringInspectionInput::IsEmpty { result, .. } => {
            Plan::new(K::KnownMember(JavaKnownMethod::StringIsEmpty, 0), result)
        }
        JavaStringInspectionInput::IndexOfLiteral { result, .. } => Plan::new(
            K::RuntimeCall(JavaRuntimeCallable::StringIndexOfLiteral, 2),
            result,
        ),
        JavaStringInspectionInput::Contains { result, .. } => {
            Plan::new(K::KnownMember(JavaKnownMethod::StringContains, 1), result)
        }
        JavaStringInspectionInput::StartsWith { result, .. } => {
            Plan::new(K::KnownMember(JavaKnownMethod::StringStartsWith, 1), result)
        }
        JavaStringInspectionInput::EndsWith { result, .. } => {
            Plan::new(K::KnownMember(JavaKnownMethod::StringEndsWith, 1), result)
        }
    })
}
