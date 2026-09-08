//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaWrappingIntegerArithmeticInput;
use crate::ast::JavaBinaryOperator;
use crate::ast::JavaUnaryOperator;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaWrappingIntegerArithmeticInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaWrappingIntegerArithmeticInput::Neg { result, .. } => {
            Plan::new(K::Unary(JavaUnaryOperator::Negate), result)
        }
        JavaWrappingIntegerArithmeticInput::Add { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::Add), result)
        }
        JavaWrappingIntegerArithmeticInput::Subtract { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::Subtract), result)
        }
        JavaWrappingIntegerArithmeticInput::Multiply { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::Multiply), result)
        }
    })
}
