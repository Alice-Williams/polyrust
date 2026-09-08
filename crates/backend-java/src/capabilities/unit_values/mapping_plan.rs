//! Exact type/value output selected from the capability input.
use super::JavaUnitValuesInput;
use crate::ast::JavaKnownType;
use crate::ast::JavaType;
use crate::capabilities::support::plans::expressions::JavaConstructionOrigin;
use crate::capabilities::support::plans::expressions::JavaExpressionNode;
use crate::capabilities::support::plans::expressions::JavaExpressionSkeleton;
use crate::capabilities::support::plans::values::JavaValuePlan;
use crate::dialect::JavaKnownConstructor;
pub type Plan = JavaValuePlan;
pub(super) fn select(
    input: &JavaUnitValuesInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaUnitValuesInput::Type => Plan::RuntimeType(JavaType::known(JavaKnownType::RuntimeUnit)),
        JavaUnitValuesInput::Value => Plan::RuntimeExpression(JavaExpressionSkeleton {
            ty: JavaType::known(JavaKnownType::RuntimeUnit),
            node: JavaExpressionNode::New {
                origin: JavaConstructionOrigin::Known(JavaKnownConstructor::RuntimeUnit),
                arguments: vec![],
            },
        }),
    })
}
