//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaFloatingPointArithmeticInput;
use crate::ast::JavaBinaryOperator;
use crate::ast::JavaUnaryOperator;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaFloatingPointArithmeticInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaFloatingPointArithmeticInput::Neg { result, .. } => {
            Plan::new(K::Unary(JavaUnaryOperator::Negate), result)
        }
        JavaFloatingPointArithmeticInput::Add { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::Add), result)
        }
        JavaFloatingPointArithmeticInput::Subtract { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::Subtract), result)
        }
        JavaFloatingPointArithmeticInput::Multiply { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::Multiply), result)
        }
        JavaFloatingPointArithmeticInput::Divide { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::Divide), result)
        }
        JavaFloatingPointArithmeticInput::Remainder { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::Remainder), result)
        }
    })
}
