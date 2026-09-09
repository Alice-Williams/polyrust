//! Prefix authority is rechecked against the current product, never stored in a path.
use super::{Engine, prefixes::Bound, state::State, values::Cell};
use crate::ast::CScalarType;
use crate::ownership::{
    CSafetyError as E,
    paths::{ElementIndex, Key, Selector, Shape},
};

impl Engine<'_, '_> {
    fn prefix_cells<'memory>(
        &self,
        path: &Key,
        memory: &'memory State,
    ) -> Result<Vec<&'memory Cell>, E> {
        memory.live(path)?;
        let Some(prefixes) = memory.prefixes.get(path.root()) else {
            return Ok(Vec::new());
        };
        let Some(Selector::Element(index)) = path.selectors().first() else {
            return Ok(Vec::new());
        };
        let mut cells = Vec::new();
        for (bound, value) in prefixes.iter() {
            if let Some(cell) = value
                && self.within_prefix(path, index, bound, memory)?
            {
                cells.push(cell);
            }
        }
        Ok(cells)
    }
    fn within_prefix(
        &self,
        path: &Key,
        index: &ElementIndex,
        bound: &Bound,
        memory: &State,
    ) -> Result<bool, E> {
        let Some(numeric) = &self.numeric else {
            return Ok(false);
        };
        let (minimum, local) = match bound {
            Bound::OriginalCount => (
                memory
                    .allocations
                    .buffer_bounds(path.root(), self.registry())?
                    .0,
                memory
                    .allocations
                    .current_count(path.root())
                    .map(|count| count.local().clone()),
            ),
            Bound::Counter { local, .. } | Bound::Snapshot(local) => {
                let Ok((minimum, _)) = numeric
                    .number(&Key::local(local), CScalarType::Size)
                    .and_then(|value| value.extent_bounds())
                else {
                    return Ok(false);
                };
                (minimum, Some(local.as_ref().clone()))
            }
        };
        if index.bounds().1 < minimum {
            return Ok(true);
        }
        Ok(match (index.source(), local) {
            (Some(source), Some(local)) => numeric
                .below_keys(&source, &Key::local(&local))
                .unwrap_or(false),
            _ => false,
        })
    }
    pub(super) fn prefix_covers(&self, path: &Key, memory: &State) -> Result<bool, E> {
        Ok(!self.prefix_cells(path, memory)?.is_empty())
    }
    pub(super) fn prefix_read(&self, path: &Key, memory: &State) -> Result<Option<Cell>, E> {
        let Shape::Elements { element, .. } = path.root().shape() else {
            return Ok(None);
        };
        let Some((_, tail)) = path.selectors().split_first() else {
            return Ok(None);
        };
        for cell in self.prefix_cells(path, memory)? {
            if let Ok(value) = cell.read(&element, tail, self.registry()) {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }
}
