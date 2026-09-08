//! Exact type/value output selected from the capability input.
use super::JavaListInput;
use crate::ast::JavaKnownType;
use crate::ast::JavaType;
use crate::capabilities::support::plans::expressions::JavaCallOrigin;
use crate::capabilities::support::plans::intrinsics::call;
use crate::capabilities::support::plans::intrinsics::holes;
use crate::capabilities::support::plans::values::JavaValuePlan;
use crate::dialect::JavaKnownCallable;
pub type Plan = JavaValuePlan;
pub(super) fn select(input: &JavaListInput) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaListInput::Type { element } => Plan::DirectType(JavaType::generic(
            JavaKnownType::List,
            vec![element.clone().boxed()],
        )),
        JavaListInput::Value { elements, result } => Plan::DirectExpression(call(
            result.clone(),
            JavaCallOrigin::Known(JavaKnownCallable::ListOf),
            false,
            holes(elements.len()),
        )),
    })
}
