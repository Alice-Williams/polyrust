//! Root containers distinguish C object cells from sparse dynamic element storage.
use super::values::Cell;
use crate::ast::{CObjectType, CRegistry};
use crate::ownership::{
    CSafetyError as E,
    paths::{ElementIndex, Key, Root, Selector, Shape},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RootCell {
    Object(Box<Cell>),
    Elements(BTreeMap<ElementIndex, Cell>),
}
impl RootCell {
    pub(super) fn object(cell: Cell) -> Self {
        Self::Object(Box::new(cell))
    }
    pub(super) fn new(root: &Root) -> Self {
        match root.shape() {
            Shape::Object(_) => Self::object(Cell::Uninitialized),
            Shape::Elements { .. } => Self::Elements(BTreeMap::new()),
        }
    }
    pub(super) fn each_mut(&mut self, mut visit: impl FnMut(&mut Cell)) {
        match self {
            Self::Object(cell) => visit(cell),
            Self::Elements(elements) => elements.values_mut().for_each(visit),
        }
    }
    pub(super) fn forget(&mut self) {
        match self {
            Self::Object(cell) => **cell = Cell::Uninitialized,
            Self::Elements(elements) => elements.clear(),
        }
    }
    pub(super) fn invalidate_index(&mut self, root: &Root) {
        if let Self::Elements(elements) = self {
            elements.retain(|index, _| !index.touches(|dependency| dependency == root));
        }
        self.each_mut(|cell| cell.invalidate_index(root));
    }
    pub(super) fn read(&self, path: &Key, registry: &CRegistry) -> Result<Cell, E> {
        match (self, path.root().shape()) {
            (Self::Object(cell), Shape::Object(ty)) => cell.read(&ty, path.selectors(), registry),
            (Self::Elements(elements), Shape::Elements { element, .. }) => {
                let (index, tail) = selection(path)?;
                if let Some((_, cell)) = elements
                    .iter()
                    .find(|(other, _)| index.join_selection(other).is_some())
                {
                    return cell.read(&element, tail, registry);
                }
                let (first, last) = index.bounds();
                // Several current bindings may identify one physical offset.
                // Coverage counts offsets, never distinct observations of one.
                let mut selected: BTreeMap<u64, Cell> = BTreeMap::new();
                for (index, cell) in elements {
                    if let Some(offset) = index.constant_value()
                        && offset >= first
                        && offset <= last
                    {
                        let joined = match selected.remove(&offset) {
                            Some(old) => old.join(cell, &element, registry)?,
                            None => cell.clone(),
                        };
                        selected.insert(offset, joined);
                    }
                }
                if selected.len() as u128 != u128::from(last) - u128::from(first) + 1 {
                    return Err(E::UninitializedStorage);
                }
                let mut result: Option<Cell> = None;
                let ty = path.ty();
                for cell in selected.into_values() {
                    let cell = cell.read(&element, tail, registry)?;
                    result = Some(match result {
                        None => cell,
                        Some(old) => old.join(&cell, &ty, registry)?,
                    });
                }
                result.ok_or(E::UninitializedStorage)
            }
            _ => Err(E::StorageTypeMismatch),
        }
    }
    pub(super) fn write(
        &mut self,
        path: &Key,
        value: &Cell,
        registry: &CRegistry,
    ) -> Result<(), E> {
        match (self, path.root().shape()) {
            (Self::Object(cell), Shape::Object(ty)) => {
                cell.write(&ty, path.selectors(), value, registry)
            }
            (Self::Elements(elements), Shape::Elements { element, .. }) => {
                let (index, tail) = selection(path)?;
                let previous: Vec<_> = elements
                    .keys()
                    .filter(|other| index.join_selection(other).is_some())
                    .cloned()
                    .collect();
                let mut selected: Option<Cell> = None;
                for previous in previous {
                    let old = elements.remove(&previous).expect("observed key");
                    selected = Some(match selected {
                        None => old,
                        Some(cell) => cell.join(&old, &element, registry)?,
                    });
                }
                for (other, cell) in elements.iter_mut() {
                    if other.overlaps(index) {
                        let mut changed = cell.clone();
                        changed.write(&element, tail, value, registry)?;
                        *cell = cell.join(&changed, &element, registry)?;
                    }
                }
                if index.exact() {
                    let mut cell = selected.unwrap_or(Cell::Uninitialized);
                    cell.write(&element, tail, value, registry)?;
                    elements.insert(index.clone(), cell);
                }
                Ok(())
            }
            _ => Err(E::StorageTypeMismatch),
        }
    }
    pub(super) fn join(
        &self,
        other: &Self,
        shape: &Shape,
        registry: &CRegistry,
    ) -> Result<Self, E> {
        match (self, other, shape) {
            (Self::Object(left), Self::Object(right), Shape::Object(ty)) => {
                left.join(right, ty, registry).map(Self::object)
            }
            (Self::Elements(left), Self::Elements(right), Shape::Elements { element, .. }) => {
                join_elements(left, right, element, registry).map(Self::Elements)
            }
            _ => Err(E::StorageTypeMismatch),
        }
    }
}

fn selection(path: &Key) -> Result<(&ElementIndex, &[Selector]), E> {
    let Some((Selector::Element(index), tail)) = path.selectors().split_first() else {
        return Err(E::UnprovedStorage);
    };
    Ok((index, tail))
}
fn join_elements(
    left: &BTreeMap<ElementIndex, Cell>,
    right: &BTreeMap<ElementIndex, Cell>,
    element: &CObjectType,
    registry: &CRegistry,
) -> Result<BTreeMap<ElementIndex, Cell>, E> {
    let mut result = BTreeMap::new();
    for (left_index, left) in left {
        for (right_index, right) in right {
            if let Some(index) = left_index.join_selection(right_index) {
                let joined = left.join(right, element, registry)?;
                let joined = match result.remove(&index) {
                    Some(old) => Cell::join(&old, &joined, element, registry)?,
                    None => joined,
                };
                result.insert(index, joined);
            }
        }
    }
    Ok(result)
}
