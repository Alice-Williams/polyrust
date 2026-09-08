//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaIntegerBitwiseInput;
use crate::ast::JavaBinaryOperator;
use crate::ast::JavaUnaryOperator;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaIntegerBitwiseInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaIntegerBitwiseInput::Not { result, .. } => {
            Plan::new(K::Unary(JavaUnaryOperator::BitNot), result)
        }
        JavaIntegerBitwiseInput::And { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::BitAnd), result)
        }
        JavaIntegerBitwiseInput::Or { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::BitOr), result)
        }
        JavaIntegerBitwiseInput::Xor { result, .. } => {
            Plan::new(K::Binary(JavaBinaryOperator::BitXor), result)
        }
    })
}
