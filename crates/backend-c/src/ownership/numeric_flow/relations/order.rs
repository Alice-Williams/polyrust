//! Strict ordering witnesses constrain current reads, never an unrelated count.
use super::{B, E, Engine, Key, Root, State, Term};
use crate::ast::{CBufferCountRef, CScalarType, CValue};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Orders<'a>(BTreeMap<(Term, Term), Vec<&'a CValue>>);
impl State<'_> {
    pub(in crate::ownership) fn below_keys(&self, left: &Key, right: &Key) -> Result<bool, E> {
        self.number(left, CScalarType::Size)?.extent_bounds()?;
        self.number(right, CScalarType::Size)?.extent_bounds()?;
        Ok(self.relations.orders.0.contains_key(&(
            Term::Read(Box::new(left.clone())),
            Term::Read(Box::new(right.clone())),
        )))
    }
}
impl<'a> Orders<'a> {
    pub(super) fn invalidate(&mut self, mut test: impl FnMut(&Root) -> bool) {
        self.0.retain(|(left, right), _| {
            ![left, right].iter().any(|term| match term {
                Term::Read(key) => key.touches(&mut test),
                Term::Constant(_) => false,
            })
        });
    }
    pub(super) fn join(&self, other: &Self) -> Self {
        let mut result = Self::default();
        for (terms, left) in &self.0 {
            if let Some(right) = other.0.get(terms) {
                let mut witnesses = left.clone();
                for witness in right {
                    if !witnesses.iter().any(|old| std::ptr::eq(*old, *witness)) {
                        witnesses.push(witness);
                    }
                }
                result.0.insert(terms.clone(), witnesses);
            }
        }
        result
    }
}
impl<'a> Engine<'a, '_> {
    pub(in crate::ownership) fn index_binding(
        &mut self,
        value: &'a CValue,
        state: &State<'a>,
    ) -> Result<Option<Key>, E> {
        Ok(match self.size_term(value, state)? {
            Some(Term::Read(key)) => Some(*key),
            _ => None,
        })
    }
    pub(super) fn remember_order(
        &mut self,
        state: &mut State<'a>,
        guard: &'a CValue,
        operator: B,
        operands: [&'a CValue; 2],
    ) -> Result<(), E> {
        let [left, right] = match operator {
            B::Less => operands,
            B::Greater => [operands[1], operands[0]],
            _ => return Ok(()),
        };
        if let (Some(left), Some(right)) =
            (self.size_term(left, state)?, self.size_term(right, state)?)
        {
            let witnesses = state.relations.orders.0.entry((left, right)).or_default();
            if !witnesses.iter().any(|old| std::ptr::eq(*old, guard)) {
                witnesses.push(guard);
            }
        }
        Ok(())
    }
    pub(in crate::ownership) fn below_count(
        &mut self,
        index: &'a CValue,
        count: &CBufferCountRef,
        state: &State<'a>,
    ) -> Result<bool, E> {
        self.numeric(index, &mut state.clone())?.extent_bounds()?;
        state
            .number(&Key::local(count.local()), CScalarType::Size)?
            .extent_bounds()?;
        let Some(left) = self.size_term(index, state)? else {
            return Ok(false);
        };
        let right = Term::Read(Box::new(Key::local(count.local())));
        Ok(state.relations.orders.0.contains_key(&(left, right)))
    }
}
