//! Exact type/value output selected from the capability input.
use super::JavaI64ValuesInput;
use crate::ast::JavaLiteral;
use crate::ast::JavaPrimitive;
use crate::ast::JavaType;
use crate::capabilities::support::plans::expressions::JavaExpressionNode;
use crate::capabilities::support::plans::expressions::JavaExpressionSkeleton;
use crate::capabilities::support::plans::values::JavaValuePlan;
pub type Plan = JavaValuePlan;
pub(super) fn select(
    input: &JavaI64ValuesInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaI64ValuesInput::Type => Plan::DirectType(JavaType::primitive(JavaPrimitive::Long)),
        JavaI64ValuesInput::Value(value) => Plan::DirectExpression(JavaExpressionSkeleton {
            ty: JavaType::primitive(JavaPrimitive::Long),
            node: JavaExpressionNode::Literal(JavaLiteral::I64(*value)),
        }),
    })
}
