//! Exact type/value output selected from the capability input.
use super::JavaResultInput;
use crate::ast::JavaKnownType;
use crate::ast::JavaType;
use crate::capabilities::support::plans::expressions::JavaCallOrigin;
use crate::capabilities::support::plans::intrinsics::call;
use crate::capabilities::support::plans::intrinsics::holes;
use crate::capabilities::support::plans::values::JavaValuePlan;
use crate::dialect::JavaRuntimeCallable;
pub type Plan = JavaValuePlan;
pub(super) fn select(
    input: &JavaResultInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaResultInput::Type { ok, error } => Plan::TaggedType(JavaType::generic(
            JavaKnownType::RuntimeValueResult,
            vec![ok.clone().boxed(), error.clone().boxed()],
        )),
        JavaResultInput::Ok { result, .. } => Plan::TaggedExpression(call(
            result.clone(),
            JavaCallOrigin::Runtime(JavaRuntimeCallable::ValueResultOk),
            false,
            holes(1),
        )),
        JavaResultInput::Err { result, .. } => Plan::TaggedExpression(call(
            result.clone(),
            JavaCallOrigin::Runtime(JavaRuntimeCallable::ValueResultErr),
            false,
            holes(1),
        )),
    })
}
