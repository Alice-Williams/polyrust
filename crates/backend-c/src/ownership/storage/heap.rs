//! Heap bindings preserve the proved storage shape across every restored alias.
use super::{
    Engine,
    state::State,
    values::{Cell, Pointer},
};
use crate::ast::{CAllocationRef, CConversion, CRegistry, CValue, CValueKind};
use crate::ownership::{
    CSafetyError as E,
    paths::{Key, Root, Shape},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum Binding {
    #[default]
    Unbound,
    Bound(Shape),
    Conflicting,
}
impl Binding {
    pub(super) fn shape(
        &self,
        requested: &Shape,
        registry: &CRegistry,
        establish: bool,
    ) -> Result<Shape, E> {
        match self {
            Self::Unbound if establish => Ok(requested.clone()),
            Self::Bound(shape) if compatible(shape, requested, registry)? => Ok(shape.clone()),
            Self::Unbound => Err(E::UnprovedAllocation),
            Self::Bound(_) | Self::Conflicting => Err(E::StorageTypeMismatch),
        }
    }
    pub(super) fn join(&self, other: &Self) -> Self {
        if self == other {
            return self.clone();
        }
        match (self, other) {
            (Self::Unbound, bound) | (bound, Self::Unbound) => bound.clone(),
            _ => Self::Conflicting,
        }
    }
}

fn compatible(left: &Shape, right: &Shape, registry: &CRegistry) -> Result<bool, E> {
    Ok(match (left, right) {
        (Shape::Object(left), Shape::Object(right)) => registry.pointee_types_match(left, right)?,
        (
            Shape::Elements {
                element: left,
                count: a,
            },
            Shape::Elements {
                element: right,
                count: b,
            },
        ) => a == b && registry.pointee_types_match(left, right)?,
        _ => false,
    })
}

impl<'ast> Engine<'_, 'ast> {
    pub(super) fn heap_root(
        &mut self,
        allocation: &CAllocationRef,
        operand: &'ast CValue,
        state: &State,
        establish: bool,
    ) -> Result<Root, E> {
        let pointer = self.expression(operand, state)?.pointer()?;
        let origin = match pointer {
            Pointer::Allocation(origin) => *origin,
            Pointer::Target(path) if path.allocation_base() => {
                let Root::Allocation(origin, _) = path.root() else {
                    return Err(E::UnprovedAllocation);
                };
                state.live(&path)?;
                origin.as_ref().clone()
            }
            Pointer::Null => return Err(E::NullStorage),
            _ => return Err(E::UnprovedAllocation),
        };
        state
            .allocations
            .restored_root(&origin, allocation, self.registry(), establish)
    }

    pub(super) fn establish_heap(
        &mut self,
        value: &'ast CValue,
        state: &mut State,
    ) -> Result<Option<Cell>, E> {
        let CValueKind::Convert {
            conversion: CConversion::AllocationRestore(allocation),
            operand,
        } = value.kind()
        else {
            return Ok(None);
        };
        let root = self.heap_root(allocation, operand, state, true)?;
        state.allocations.bind(&root)?;
        state
            .roots
            .entry(root.clone())
            .or_insert_with(|| super::root_cells::RootCell::new(&root));
        Ok(Some(Cell::Pointer(Pointer::Target(Box::new(
            Key::from_root(root),
        )))))
    }
}
