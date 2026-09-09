//! A false ordered comparison may come from either operand being NaN.
use super::{B, E, Engine, NumericDomain, State, negate, reverse};
use crate::ast::{CScalarType, CValue};

impl<'a> Engine<'a, '_> {
    pub(super) fn float_compare(
        &mut self,
        mut state: State<'a>,
        operator: B,
        operands: [(&'a CValue, &NumericDomain); 2],
        truth: bool,
    ) -> Result<Option<State<'a>>, E> {
        let [(left, lhs), (right, rhs)] = operands;
        let a = lhs.convert(CScalarType::F64)?.domain;
        let b = rhs.convert(CScalarType::F64)?.domain;
        if lhs.ty() == CScalarType::F64 {
            let domain = restriction(lhs, &b, operator, truth)?;
            if !self.restrict_value(&mut state, left, domain)? {
                return Ok(None);
            }
        }
        if rhs.ty() == CScalarType::F64 {
            let domain = restriction(rhs, &a, reverse(operator), truth)?;
            if !self.restrict_value(&mut state, right, domain)? {
                return Ok(None);
            }
        }
        Ok(Some(state))
    }
}

fn restriction(
    value: &NumericDomain,
    bound: &NumericDomain,
    operator: B,
    truth: bool,
) -> Result<NumericDomain, E> {
    // NaN on the other side alone can explain false ordered/Equal, or true
    // NotEqual; those paths cannot restrict this operand at all.
    let nan_explains = match operator {
        B::NotEqual => truth,
        _ => !truth,
    };
    if nan_explains && bound.may_nan() {
        return Ok(value.clone());
    }
    let effective = if truth { operator } else { negate(operator) };
    let allow_nan = nan_explains;
    let Some((min, max)) = bound.floating_bounds() else {
        return Ok(if allow_nan {
            value.clone()
        } else {
            NumericDomain::empty(CScalarType::F64)
        });
    };
    match effective {
        B::Less if max == f64::NEG_INFINITY => value.restrict_float(1.0, 0.0, allow_nan),
        B::Greater if min == f64::INFINITY => value.restrict_float(1.0, 0.0, allow_nan),
        B::Less => value.restrict_float(f64::NEG_INFINITY, max.next_down(), allow_nan),
        B::LessEqual => value.restrict_float(f64::NEG_INFINITY, max, allow_nan),
        B::Greater => value.restrict_float(min.next_up(), f64::INFINITY, allow_nan),
        B::GreaterEqual => value.restrict_float(min, f64::INFINITY, allow_nan),
        B::Equal => value.restrict_float(min, max, false),
        // Keeping an interior excluded floating value is optional precision;
        // retaining it cannot justify an unsafe cast or integer operation.
        _ => Ok(value.clone()),
    }
}
