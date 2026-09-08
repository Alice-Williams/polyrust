//! Exact type/value output selected from the capability input.
use super::JavaBoolValuesInput;
use crate::ast::JavaLiteral;
use crate::ast::JavaPrimitive;
use crate::ast::JavaType;
use crate::capabilities::support::plans::expressions::JavaExpressionNode;
use crate::capabilities::support::plans::expressions::JavaExpressionSkeleton;
use crate::capabilities::support::plans::values::JavaValuePlan;
pub type Plan = JavaValuePlan;
pub(super) fn select(
    input: &JavaBoolValuesInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaBoolValuesInput::Type => Plan::DirectType(JavaType::primitive(JavaPrimitive::Boolean)),
        JavaBoolValuesInput::Value(value) => Plan::DirectExpression(JavaExpressionSkeleton {
            ty: JavaType::primitive(JavaPrimitive::Boolean),
            node: JavaExpressionNode::Literal(JavaLiteral::Boolean(*value)),
        }),
    })
}
