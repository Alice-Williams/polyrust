//! Exact type/value output selected from the capability input.
use super::JavaF64ValuesInput;
use crate::ast::JavaLiteral;
use crate::ast::JavaPrimitive;
use crate::ast::JavaType;
use crate::capabilities::support::plans::expressions::JavaCallOrigin;
use crate::capabilities::support::plans::expressions::JavaExpressionNode;
use crate::capabilities::support::plans::expressions::JavaExpressionSkeleton;
use crate::capabilities::support::plans::intrinsics::call;
use crate::capabilities::support::plans::intrinsics::owned;
use crate::capabilities::support::plans::values::JavaValuePlan;
use crate::dialect::JavaKnownCallable;
pub type Plan = JavaValuePlan;
pub(super) fn select(
    input: &JavaF64ValuesInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaF64ValuesInput::Type => Plan::DirectType(JavaType::primitive(JavaPrimitive::Double)),
        JavaF64ValuesInput::Value(value) => Plan::DirectExpression(call(
            JavaType::primitive(JavaPrimitive::Double),
            JavaCallOrigin::Known(JavaKnownCallable::DoubleFromLongBits),
            false,
            vec![owned(JavaExpressionSkeleton {
                ty: JavaType::primitive(JavaPrimitive::Long),
                node: JavaExpressionNode::Literal(JavaLiteral::I64(*value as i64)),
            })],
        )),
    })
}
