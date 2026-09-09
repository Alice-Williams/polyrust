//! Recognize only value-preserving size terms and guards over their current values.
use super::{Arithmetic, B, E, Engine, Relation, State, Term, unsigned_size};
use crate::ast::{CConversion, CValue, CValueKind as V};
use crate::ownership::{
    constants,
    numeric_flow::storage::{self, Key},
};

impl<'a> Engine<'a> {
    pub(super) fn size_term(&mut self, value: &CValue) -> Result<Option<Term>, E> {
        if !storage::scalar(self.registry, value.ty())?.is_some_and(unsigned_size) {
            return Ok(None);
        }
        if let Ok(number) = constants::evaluate(&mut self.layouts, value)
            && let Ok(integer) = number.integer()
            && let Ok(value) = u64::try_from(integer.value())
        {
            return Ok(Some(Term::Constant(value)));
        }
        match value.kind() {
            V::Read(place) => {
                Ok(Key::place(place, &mut self.layouts).map(|key| Term::Read(Box::new(key))))
            }
            V::Convert {
                conversion: CConversion::Numeric(_),
                operand,
            } => self.size_term(operand),
            _ => Ok(None),
        }
    }
    pub(in crate::ownership::numeric_flow) fn remember_size_guard(
        &mut self,
        state: &mut State<'a>,
        guard: &'a CValue,
        operator: B,
        operands: [&'a CValue; 2],
    ) -> Result<(), E> {
        let [left, right] = operands;
        for (operator, left, right) in [
            (operator, left, right),
            (super::super::refine::reverse(operator), right, left),
        ] {
            if !matches!(operator, B::Less | B::LessEqual) {
                continue;
            }
            let V::Binary {
                operator,
                left: maximum,
                right: bound,
            } = right.kind()
            else {
                continue;
            };
            let operation = match operator {
                B::Subtract => Arithmetic::Add,
                B::Divide => Arithmetic::Multiply,
                _ => continue,
            };
            if self.size_term(maximum)? != Some(Term::Constant(u64::MAX)) {
                continue;
            }
            let (Some(left), Some(right)) = (self.size_term(left)?, self.size_term(bound)?) else {
                continue;
            };
            if operation == Arithmetic::Multiply
                && self
                    .numeric(bound, state)?
                    .domain
                    .integer_bounds()
                    .is_none_or(|(min, _)| min < 1)
            {
                continue;
            }
            let witnesses = state
                .relations
                .guards
                .entry(Relation::new(operation, left, right))
                .or_default();
            if !witnesses.iter().any(|old| std::ptr::eq(*old, guard)) {
                witnesses.push(guard);
            }
        }
        Ok(())
    }
}
