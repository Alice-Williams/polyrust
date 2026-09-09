//! Buffer offsets use the original count and current, invalidatable index bindings.
use super::{Engine, state::State};
use crate::ast::CValue;
use crate::ownership::{
    CSafetyError as E,
    paths::{ElementIndex, Key, Selector},
};

impl<'ast> Engine<'_, 'ast> {
    pub(super) fn buffer_offset(
        &self,
        base: Key,
        index: &'ast CValue,
        bounds: (u64, u64),
        state: &State,
    ) -> Result<Key, E> {
        let [Selector::Element(origin)] = base.selectors() else {
            return Err(E::UnprovedStorage);
        };
        let (minimum, _) = state
            .allocations
            .buffer_bounds(base.root(), self.registry())?;
        let original_base = origin.constant_value() == Some(0);
        let source = if original_base {
            self.index_binding(index, state)?
        } else {
            None
        };
        if bounds == (0, 0) && source.is_none() {
            return Ok(base);
        }
        let (first, last) = origin.bounds();
        let first = first.checked_add(bounds.0).ok_or(E::IndexOutOfBounds)?;
        let last = last.checked_add(bounds.1).ok_or(E::IndexOutOfBounds)?;
        let relational = if last >= minimum && original_base {
            match state.allocations.current_count(base.root()) {
                Some(count) => self.index_below_count(index, &count, state)?,
                None => false,
            }
        } else {
            false
        };
        if last >= minimum && !relational {
            return Err(E::IndexOutOfBounds);
        }
        base.element_selection(ElementIndex::checked(first, last, source.as_ref())?)
            .ok_or(E::UnprovedStorage)
    }
}
