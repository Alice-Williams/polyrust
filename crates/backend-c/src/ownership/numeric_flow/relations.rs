//! Algebraic bounds retain actual guard witnesses and authenticated storage terms.
mod derive;

use super::{
    DomainTransfer, E, Engine, Number, NumericDomain, NumericLoss, State,
    storage::{Key, Root},
};
use crate::ast::{CBinaryOperator as B, CScalarType, CValue};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Term {
    Read(Box<Key>),
    Constant(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Arithmetic {
    Add,
    Multiply,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Relation {
    operation: Arithmetic,
    left: Term,
    right: Term,
}
impl Relation {
    fn new(operation: Arithmetic, left: Term, right: Term) -> Self {
        let (left, right) = if left <= right {
            (left, right)
        } else {
            (right, left)
        };
        Self {
            operation,
            left,
            right,
        }
    }
    fn touches(&self, mut test: impl FnMut(&Root) -> bool) -> bool {
        [&self.left, &self.right].iter().any(|term| match term {
            Term::Read(key) => test(key.root()),
            Term::Constant(_) => false,
        })
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Relations<'a> {
    guards: BTreeMap<Relation, Vec<&'a CValue>>,
}
impl<'a> Relations<'a> {
    pub(super) fn invalidate(&mut self, mut test: impl FnMut(&Root) -> bool) {
        self.guards
            .retain(|relation, _| !relation.touches(&mut test));
    }
    pub(super) fn join(&self, other: &Self) -> Self {
        let mut joined = Self::default();
        for (relation, witnesses) in &self.guards {
            if let Some(right) = other.guards.get(relation) {
                let mut witnesses = witnesses.clone();
                for witness in right {
                    if !witnesses.iter().any(|old| std::ptr::eq(*old, *witness)) {
                        witnesses.push(witness);
                    }
                }
                joined.guards.insert(relation.clone(), witnesses);
            }
        }
        joined
    }
}

impl<'a> Engine<'a> {
    pub(super) fn bounded_binary(
        &mut self,
        operator: B,
        operands: [(&CValue, &Number<'a>); 2],
        state: &State<'a>,
    ) -> Result<DomainTransfer, E> {
        let [(left, lhs), (right, rhs)] = operands;
        let mut result = lhs.domain.binary(operator, &rhs.domain)?;
        let operation = match operator {
            B::Add => Arithmetic::Add,
            B::Multiply => Arithmetic::Multiply,
            _ => return Ok(result),
        };
        let (Some(left), Some(right)) = (self.size_term(left)?, self.size_term(right)?) else {
            return Ok(result);
        };
        if !state
            .relations
            .guards
            .contains_key(&Relation::new(operation, left, right))
        {
            return Ok(result);
        }
        let (Some((a, _)), Some((b, _))) =
            (lhs.domain.integer_bounds(), rhs.domain.integer_bounds())
        else {
            return Ok(result);
        };
        let lower = match operation {
            Arithmetic::Add => a as u128 + b as u128,
            Arithmetic::Multiply => a as u128 * b as u128,
        };
        if lower > u128::from(u64::MAX) {
            return Err(E::UnprovedSizeArithmetic);
        }
        result.domain = NumericDomain::full(result.domain.ty())?
            .restrict_integer(lower as i128, i128::from(u64::MAX))?;
        result.loss = NumericLoss::None;
        Ok(result)
    }
}

fn unsigned_size(ty: CScalarType) -> bool {
    matches!(ty, CScalarType::Size | CScalarType::U64)
}
