//! Exact type/value output selected from the capability input.
use super::JavaBytesInput;
use crate::ast::JavaKnownType;
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
use crate::dialect::JavaRuntimeCallable;
pub type Plan = JavaValuePlan;
pub(super) fn select(
    input: &JavaBytesInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaBytesInput::Type => Plan::RuntimeType(JavaType::known(JavaKnownType::RuntimeBytes)),
        JavaBytesInput::Value { values, result } => Plan::RuntimeExpression(call(
            result.clone(),
            JavaCallOrigin::Runtime(JavaRuntimeCallable::BytesOf),
            false,
            vec![owned(call(
                JavaType::generic(
                    JavaKnownType::List,
                    vec![JavaType::Boxed(JavaPrimitive::Int)],
                ),
                JavaCallOrigin::Known(JavaKnownCallable::ListOf),
                false,
                values
                    .iter()
                    .map(|value| {
                        owned(JavaExpressionSkeleton {
                            ty: JavaType::primitive(JavaPrimitive::Int),
                            node: JavaExpressionNode::Literal(JavaLiteral::I32(i32::from(*value))),
                        })
                    })
                    .collect(),
            ))],
        )),
    })
}
