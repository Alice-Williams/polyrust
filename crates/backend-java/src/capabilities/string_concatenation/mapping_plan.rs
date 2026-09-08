//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaStringConcatenationInput;
use crate::ast::JavaBinaryOperator;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaStringConcatenationInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(Plan::new(K::Binary(JavaBinaryOperator::Add), &input.result))
}
