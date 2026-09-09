//! Inductive coverage is not a public initialization flag or a C array type.
use super::values::Cell;
use crate::ast::{CLocalRef, CLoopRef, CObjectType, CRegistry};
use crate::ownership::{
    CSafetyError as E,
    paths::{Key, Root, Selector},
};
use std::collections::BTreeMap;
#[cfg(test)]
#[path = "../../tests/buffer_prefix_lattice.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Bound {
    Counter {
        identity: Box<CLoopRef>,
        local: Box<CLocalRef>,
    },
    Snapshot(Box<CLocalRef>),
    OriginalCount,
}
impl Bound {
    pub(super) fn local(&self) -> Option<&CLocalRef> {
        match self {
            Self::Counter { local, .. } | Self::Snapshot(local) => Some(local),
            Self::OriginalCount => None,
        }
    }
}

/// None is a proved empty prefix; an absent map entry is no evidence.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Prefixes(BTreeMap<Bound, Option<Cell>>);
impl Prefixes {
    pub(super) fn get(&self, bound: &Bound) -> Option<&Option<Cell>> {
        self.0.get(bound)
    }
    pub(super) fn insert(&mut self, bound: Bound, value: Option<Cell>) {
        self.0.insert(bound, value);
    }
    pub(super) fn iter(&self) -> impl Iterator<Item = (&Bound, &Option<Cell>)> {
        self.0.iter()
    }
    pub(super) fn each_mut(&mut self, mut visit: impl FnMut(&mut Cell)) {
        for cell in self.0.values_mut().flatten() {
            visit(cell);
        }
    }
    pub(super) fn invalidate_index(&mut self, root: &Root) {
        self.0.retain(|bound, _| {
            bound
                .local()
                .is_none_or(|local| root != &Root::Local(local.clone()))
        });
        self.each_mut(|cell| cell.invalidate_index(root));
    }
    pub(super) fn write(
        &mut self,
        path: &Key,
        value: &Cell,
        element: &CObjectType,
        registry: &CRegistry,
    ) -> Result<(), E> {
        let Some((Selector::Element(index), tail)) = path.selectors().split_first() else {
            return Err(E::UnprovedStorage);
        };
        for (bound, summary) in &mut self.0 {
            // The frontier itself is outside the half-open initialized prefix.
            if bound
                .local()
                .is_some_and(|local| index.source() == Some(Key::local(local)))
            {
                continue;
            }
            if let Some(old) = summary {
                let mut changed = old.clone();
                changed.write(element, tail, value, registry)?;
                *old = old.join(&changed, element, registry)?;
            }
        }
        Ok(())
    }
    pub(super) fn join(
        &self,
        other: &Self,
        element: &CObjectType,
        registry: &CRegistry,
    ) -> Result<Self, E> {
        let mut result = Self::default();
        for (bound, left) in &self.0 {
            if let Some(right) = other.0.get(bound) {
                result.insert(bound.clone(), join_values(left, right, element, registry)?);
            }
        }
        Ok(result)
    }
}

pub(super) fn join_values(
    left: &Option<Cell>,
    right: &Option<Cell>,
    element: &CObjectType,
    registry: &CRegistry,
) -> Result<Option<Cell>, E> {
    Ok(match (left, right) {
        (None, value) | (value, None) => value.clone(),
        (Some(left), Some(right)) => Some(left.join(right, element, registry)?),
    })
}
