//! Predicate refinement is derived from actual children and retained dependencies.
mod comparisons;
mod floating;

use super::state::NaNPolarity;
use super::{E, Engine, NumericDomain, State};
use crate::ast::{
    CBinaryOperator as B, CConversion, CScalarType, CUnaryOperator, CValue, CValueKind as V,
};

impl<'a> Engine<'a, '_> {
    pub(in crate::ownership) fn refine(
        &mut self,
        state: &State<'a>,
        value: &'a CValue,
        truth: bool,
    ) -> Result<Option<State<'a>>, E> {
        let mut refined = state.clone();
        let number = self.numeric(value, &mut refined)?;
        if number.domain.is_empty() || number.domain.truth().is_some_and(|known| known != truth) {
            return Ok(None);
        }
        match value.kind() {
            V::Convert {
                conversion: CConversion::Numeric(CScalarType::Bool),
                operand,
            } => return self.refine(&refined, operand, truth),
            V::Unary {
                operator: CUnaryOperator::LogicalNot,
                operand,
            } => return self.refine(&refined, operand, !truth),
            V::Binary {
                operator: B::LogicalAnd,
                left,
                right,
            } => return self.logical_refine(&refined, left, right, true, truth),
            V::Binary {
                operator: B::LogicalOr,
                left,
                right,
            } => return self.logical_refine(&refined, left, right, false, truth),
            V::Binary {
                operator,
                left,
                right,
            } if matches!(
                operator,
                B::Equal | B::NotEqual | B::Less | B::LessEqual | B::Greater | B::GreaterEqual
            ) =>
            {
                let mut result = self.compare_refine(refined, *operator, left, right, truth)?;
                if let Some(state) = &mut result {
                    self.remember_size_guard(
                        state,
                        value,
                        if truth { *operator } else { negate(*operator) },
                        [left, right],
                    )?;
                }
                return Ok(result);
            }
            _ => {}
        }
        if let Some(predicate) = &number.predicate {
            let mut operand_state = refined.clone();
            // Reconstructing a retained predicate is proof work, not a new
            // execution of the original expression at this graph point.
            let previous = self.mode;
            if matches!(previous, super::Mode::Verify(_)) {
                self.mode = super::Mode::Derive;
            }
            let operand = self.numeric(predicate.operand, &mut operand_state);
            self.mode = previous;
            let operand = operand?;
            let nan = truth == (predicate.polarity == NaNPolarity::Nonzero);
            let domain = if nan {
                operand.domain.only_nan()?
            } else {
                operand
                    .domain
                    .restrict_float(f64::NEG_INFINITY, f64::INFINITY, false)?
            };
            if !self.restrict_value(&mut refined, predicate.operand, domain)? {
                return Ok(None);
            }
        }
        let domain = if number.domain.integer_bounds().is_some() {
            if truth {
                number.domain.exclude_integer(0)?
            } else {
                number.domain.restrict_integer(0, 0)?
            }
        } else if !truth {
            number.domain.restrict_float(0.0, 0.0, false)?
        } else {
            number.domain
        };
        if !self.restrict_value(&mut refined, value, domain)? {
            return Ok(None);
        }
        Ok(Some(refined))
    }

    fn logical_refine(
        &mut self,
        state: &State<'a>,
        left: &'a CValue,
        right: &'a CValue,
        and: bool,
        truth: bool,
    ) -> Result<Option<State<'a>>, E> {
        if truth == and {
            let Some(left) = self.refine(state, left, truth)? else {
                return Ok(None);
            };
            self.refine(&left, right, truth)
        } else {
            let short = self.refine(state, left, truth)?;
            let selected = match self.refine(state, left, !truth)? {
                Some(left) => self.refine(&left, right, truth)?,
                None => None,
            };
            match (short, selected) {
                (Some(left), Some(right)) => left.join(&right, false).map(Some),
                (left, right) => Ok(left.or(right)),
            }
        }
    }

    pub(super) fn restrict_value(
        &mut self,
        state: &mut State<'a>,
        value: &'a CValue,
        domain: NumericDomain,
    ) -> Result<bool, E> {
        if domain.is_empty() {
            return Ok(false);
        }
        if let V::Read(place) = value.kind()
            && let Some(key) = self.exact_place(place, state)?
        {
            let mut number = state.number(&key, domain.ty())?;
            number.domain = number.domain.intersect(&domain)?;
            if number.domain.is_empty() {
                return Ok(false);
            }
            state.set(key, number);
        }
        Ok(true)
    }
}

pub(super) fn reverse(operator: B) -> B {
    match operator {
        B::Less => B::Greater,
        B::LessEqual => B::GreaterEqual,
        B::Greater => B::Less,
        B::GreaterEqual => B::LessEqual,
        operator => operator,
    }
}
fn negate(operator: B) -> B {
    match operator {
        B::Less => B::GreaterEqual,
        B::LessEqual => B::Greater,
        B::Greater => B::LessEqual,
        B::GreaterEqual => B::Less,
        B::Equal => B::NotEqual,
        B::NotEqual => B::Equal,
        operator => operator,
    }
}
