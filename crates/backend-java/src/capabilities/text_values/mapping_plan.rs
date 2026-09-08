//! Exact type/value output selected from the capability input.
use super::JavaTextValuesInput;
use crate::ast::JavaKnownType;
use crate::ast::JavaLiteral;
use crate::ast::JavaType;
use crate::capabilities::support::plans::expressions::JavaExpressionNode;
use crate::capabilities::support::plans::expressions::JavaExpressionSkeleton;
use crate::capabilities::support::plans::values::JavaValuePlan;
pub type Plan = JavaValuePlan;
pub(super) fn select(
    input: &JavaTextValuesInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaTextValuesInput::Type => Plan::DirectType(JavaType::known(JavaKnownType::String)),
        JavaTextValuesInput::Value(value) => Plan::DirectExpression(JavaExpressionSkeleton {
            ty: JavaType::known(JavaKnownType::String),
            node: JavaExpressionNode::Literal(JavaLiteral::String(value.clone())),
        }),
    })
}
