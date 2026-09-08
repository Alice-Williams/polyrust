//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaOrderingInput;
use crate::ast::JavaBinaryOperator;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaOrderingInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    let (operator, left, result) = match input {
        JavaOrderingInput::Less { left, result, .. } => (JavaBinaryOperator::Less, left, result),
        JavaOrderingInput::LessEqual { left, result, .. } => {
            (JavaBinaryOperator::LessEqual, left, result)
        }
        JavaOrderingInput::Greater { left, result, .. } => {
            (JavaBinaryOperator::Greater, left, result)
        }
        JavaOrderingInput::GreaterEqual { left, result, .. } => {
            (JavaBinaryOperator::GreaterEqual, left, result)
        }
    };
    let kind = match super::domain::select(&left.ty)? {
        super::domain::JavaOrderingDomain::Numeric => K::Binary(operator),
        super::domain::JavaOrderingDomain::String => K::StringOrdering(operator),
        super::domain::JavaOrderingDomain::Scalar => K::ScalarOrdering(operator),
    };
    Ok(Plan::new(kind, result))
}
