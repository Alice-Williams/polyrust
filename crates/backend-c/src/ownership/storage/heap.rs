//! Fixed heap bindings are proof state, not another allocation or a C cast effect.
use super::{
    Engine,
    state::State,
    values::{Cell, Pointer},
};
use crate::ast::{CAllocationRef, CConversion, CObjectType, CRegistry, CValue, CValueKind};
use crate::ownership::{
    CSafetyError as E,
    paths::{Key, Root},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum Binding {
    #[default]
    Unbound,
    Object(CObjectType),
    Conflicting,
}
impl Binding {
    pub(super) fn object(
        &self,
        requested: &CObjectType,
        registry: &CRegistry,
        establish: bool,
    ) -> Result<CObjectType, E> {
        match self {
            Self::Unbound if establish => Ok(registry.pointee_storage_identity(requested)?),
            Self::Object(ty) if registry.pointee_types_match(ty, requested)? => Ok(ty.clone()),
            Self::Unbound => Err(E::UnprovedAllocation),
            Self::Object(_) | Self::Conflicting => Err(E::StorageTypeMismatch),
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
            Pointer::Target(path) if path.whole_root() => {
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
            .or_insert(Cell::Uninitialized);
        Ok(Some(Cell::Pointer(Pointer::Target(Box::new(
            Key::from_root(root),
        )))))
    }
}
