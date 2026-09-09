//! Integer preimages are restricted only when C conversions preserve values.
use super::{B, E, Engine, NumericDomain, State, negate, reverse};
use crate::ast::{CScalarType, CValue};
use crate::ownership::ranges::NumericLoss;

impl<'a> Engine<'a> {
    pub(super) fn compare_refine(
        &mut self,
        mut state: State<'a>,
        operator: B,
        left: &'a CValue,
        right: &'a CValue,
        truth: bool,
    ) -> Result<Option<State<'a>>, E> {
        let lhs = self.numeric(left, &mut state)?;
        let rhs = self.numeric(right, &mut state)?;
        let common = lhs.domain.ty().usual_arithmetic_conversion(rhs.domain.ty());
        if common == CScalarType::F64 {
            return self.float_compare(
                state,
                operator,
                [(left, &lhs.domain), (right, &rhs.domain)],
                truth,
            );
        }
        let a = lhs.domain.convert(common)?;
        let b = rhs.domain.convert(common)?;
        let operator = if truth { operator } else { negate(operator) };
        if a.loss == NumericLoss::None
            && !self.restrict_value(
                &mut state,
                left,
                integer_restriction(&lhs.domain, &b.domain, operator)?,
            )?
        {
            return Ok(None);
        }
        if b.loss == NumericLoss::None
            && !self.restrict_value(
                &mut state,
                right,
                integer_restriction(&rhs.domain, &a.domain, reverse(operator))?,
            )?
        {
            return Ok(None);
        }
        Ok(Some(state))
    }
}

fn integer_restriction(
    value: &NumericDomain,
    bound: &NumericDomain,
    operator: B,
) -> Result<NumericDomain, E> {
    let Some((min, max)) = bound.integer_bounds() else {
        return Ok(NumericDomain::empty(value.ty()));
    };
    match operator {
        B::Less => value.restrict_integer(i128::MIN, max - 1),
        B::LessEqual => value.restrict_integer(i128::MIN, max),
        B::Greater => value.restrict_integer(min + 1, i128::MAX),
        B::GreaterEqual => value.restrict_integer(min, i128::MAX),
        B::Equal => value.restrict_integer(min, max),
        B::NotEqual if min == max => value.exclude_integer(min),
        _ => Ok(value.clone()),
    }
}
