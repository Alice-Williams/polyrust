//! Exact type/value output selected from the capability input.
use super::JavaOptionInput;
use crate::ast::JavaKnownType;
use crate::ast::JavaType;
use crate::capabilities::support::plans::expressions::JavaCallOrigin;
use crate::capabilities::support::plans::intrinsics::call;
use crate::capabilities::support::plans::intrinsics::holes;
use crate::capabilities::support::plans::values::JavaValuePlan;
use crate::dialect::JavaRuntimeCallable;
pub type Plan = JavaValuePlan;
pub(super) fn select(
    input: &JavaOptionInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaOptionInput::Type { inner } => Plan::TaggedType(JavaType::generic(
            JavaKnownType::RuntimeOption,
            vec![inner.clone().boxed()],
        )),
        JavaOptionInput::None { result } => Plan::TaggedExpression(call(
            result.clone(),
            JavaCallOrigin::Runtime(JavaRuntimeCallable::OptionNone),
            false,
            holes(0),
        )),
        JavaOptionInput::Some { result, .. } => Plan::TaggedExpression(call(
            result.clone(),
            JavaCallOrigin::Runtime(JavaRuntimeCallable::OptionSome),
            false,
            holes(1),
        )),
    })
}
