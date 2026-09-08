//! Exact type/value output selected from the capability input.
use super::JavaCharValuesInput;
use crate::ast::JavaKnownType;
use crate::ast::JavaLiteral;
use crate::ast::JavaPrimitive;
use crate::ast::JavaType;
use crate::capabilities::support::plans::expressions::JavaConstructionOrigin;
use crate::capabilities::support::plans::expressions::JavaExpressionNode;
use crate::capabilities::support::plans::expressions::JavaExpressionSkeleton;
use crate::capabilities::support::plans::intrinsics::owned;
use crate::capabilities::support::plans::values::JavaValuePlan;
use crate::dialect::JavaKnownConstructor;
pub type Plan = JavaValuePlan;
pub(super) fn select(
    input: &JavaCharValuesInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaCharValuesInput::Type => {
            Plan::RuntimeType(JavaType::known(JavaKnownType::RuntimeScalar))
        }
        JavaCharValuesInput::Value(value) => Plan::RuntimeExpression(JavaExpressionSkeleton {
            ty: JavaType::known(JavaKnownType::RuntimeScalar),
            node: JavaExpressionNode::New {
                origin: JavaConstructionOrigin::Known(JavaKnownConstructor::RuntimeScalar),
                arguments: vec![owned(JavaExpressionSkeleton {
                    ty: JavaType::primitive(JavaPrimitive::Int),
                    node: JavaExpressionNode::Literal(JavaLiteral::CharScalar(u32::from(*value))),
                })],
            },
        }),
    })
}
