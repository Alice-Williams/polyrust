//! Raw default allocations: resource existence is separate from pointer copies.
#[cfg(test)]
#[path = "../../tests/allocation_lattice.rs"]
mod tests;
use super::{
    state::State,
    values::{Cell, Pointer},
};
use crate::ownership::{
    CSafetyError as E,
    numeric_flow::{AllocationOrigin, AllocationRequest},
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Status {
    Possible,
    Live,
    Null,
    Released,
    Settled,
    Unproved,
}
impl Status {
    fn outstanding(self) -> bool {
        matches!(self, Self::Possible | Self::Live | Self::Unproved)
    }
    fn join(self, other: Self) -> Self {
        if self == other {
            return self;
        }
        match (self, other) {
            (
                Self::Null | Self::Live | Self::Possible,
                Self::Null | Self::Live | Self::Possible,
            ) => Self::Possible,
            (
                Self::Null | Self::Released | Self::Settled,
                Self::Null | Self::Released | Self::Settled,
            ) => Self::Settled,
            _ => Self::Unproved,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Allocation {
    request: AllocationRequest,
    status: Status,
    binding: super::heap::Binding,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Allocations(BTreeMap<AllocationOrigin, Allocation>);
impl Allocations {
    pub(super) fn restored_root(
        &self,
        origin: &AllocationOrigin,
        descriptor: &crate::ast::CAllocationRef,
        registry: &crate::ast::CRegistry,
        establish: bool,
    ) -> Result<crate::ownership::paths::Root, E> {
        if self.nonnull(origin)? != Some(true) {
            return Err(E::NullStorage);
        }
        let allocation = self.0.get(origin).ok_or(E::UnprovedAllocation)?;
        allocation.request.admits_object(descriptor, registry)?;
        let ty = allocation
            .binding
            .object(descriptor.object_type(), registry, establish)?;
        Ok(crate::ownership::paths::Root::Allocation(
            Box::new(origin.clone()),
            ty,
        ))
    }
    pub(super) fn bind(&mut self, root: &crate::ownership::paths::Root) -> Result<(), E> {
        let crate::ownership::paths::Root::Allocation(origin, ty) = root else {
            return Err(E::UnprovedAllocation);
        };
        self.0.get_mut(origin).ok_or(E::UnprovedAllocation)?.binding =
            super::heap::Binding::Object(ty.clone());
        Ok(())
    }
    pub(super) fn start(&mut self, request: AllocationRequest) -> Result<AllocationOrigin, E> {
        request.validate()?;
        let origin = request.origin().clone();
        if self
            .0
            .get(&origin)
            .is_some_and(|old| old.status.outstanding())
        {
            return Err(E::UnreleasedAllocation);
        }
        self.0.insert(
            origin.clone(),
            Allocation {
                request,
                status: Status::Possible,
                binding: super::heap::Binding::Unbound,
            },
        );
        Ok(origin)
    }
    pub(super) fn nonnull(&self, origin: &AllocationOrigin) -> Result<Option<bool>, E> {
        match self.0.get(origin).ok_or(E::UnprovedAllocation)?.status {
            Status::Possible => Ok(None),
            Status::Live => Ok(Some(true)),
            Status::Null => Ok(Some(false)),
            Status::Released | Status::Settled | Status::Unproved => {
                Err(E::InvalidAllocationRelease)
            }
        }
    }
    pub(super) fn refine(&mut self, origin: &AllocationOrigin, nonnull: bool) -> Result<bool, E> {
        if self.nonnull(origin)?.is_some_and(|known| known != nonnull) {
            return Ok(false);
        }
        self.0.get_mut(origin).ok_or(E::UnprovedAllocation)?.status =
            if nonnull { Status::Live } else { Status::Null };
        Ok(true)
    }
    pub(super) fn release(&mut self, origin: &AllocationOrigin) -> Result<bool, E> {
        let allocation = self.0.get_mut(origin).ok_or(E::InvalidAllocationRelease)?;
        match allocation.status {
            Status::Possible | Status::Live => {
                allocation.status = Status::Released;
                Ok(true)
            }
            Status::Null => Ok(false),
            Status::Released | Status::Settled | Status::Unproved => {
                Err(E::InvalidAllocationRelease)
            }
        }
    }
    pub(super) fn finish(&self) -> Result<(), E> {
        if self.0.values().any(|value| value.status.outstanding()) {
            Err(E::UnreleasedAllocation)
        } else {
            Ok(())
        }
    }
    pub(super) fn join(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (origin, value) in &other.0 {
            result
                .0
                .entry(origin.clone())
                .and_modify(|old| {
                    old.binding = old.binding.join(&value.binding);
                    old.status = if old.request == value.request {
                        old.status.join(value.status)
                    } else {
                        Status::Unproved
                    };
                })
                .or_insert_with(|| value.clone());
        }
        result
    }
}
impl State {
    pub(super) fn expire_allocation(&mut self, origin: &AllocationOrigin) {
        self.roots.retain(|root, _| {
            !matches!(root,
            crate::ownership::paths::Root::Allocation(old, _) if old.as_ref() == origin)
        });
        for cell in self.roots.values_mut() {
            cell.expire_allocation(origin);
        }
    }
}
impl Cell {
    pub(super) fn contains_allocation(&self) -> bool {
        match self {
            Self::Pointer(Pointer::Allocation(_)) => true,
            Self::Pointer(Pointer::Target(path)) => {
                matches!(path.root(), crate::ownership::paths::Root::Allocation(..))
            }
            Self::Record(fields) => fields.values().any(Self::contains_allocation),
            Self::Array { default, elements } => {
                default.contains_allocation() || elements.values().any(Self::contains_allocation)
            }
            Self::Union { value, .. } => value.contains_allocation(),
            _ => false,
        }
    }
    fn expire_allocation(&mut self, origin: &AllocationOrigin) {
        match self {
            Self::Pointer(Pointer::Allocation(old)) if old.as_ref() == origin => {
                *self = Self::Pointer(Pointer::Expired)
            }
            Self::Pointer(Pointer::Target(path))
                if matches!(path.root(),
                crate::ownership::paths::Root::Allocation(old, _) if old.as_ref() == origin) =>
            {
                *self = Self::Pointer(Pointer::Expired)
            }
            Self::Record(fields) => {
                for cell in fields.values_mut() {
                    cell.expire_allocation(origin);
                }
            }
            Self::Array { default, elements } => {
                default.expire_allocation(origin);
                for cell in elements.values_mut() {
                    cell.expire_allocation(origin);
                }
            }
            Self::Union { value, .. } => value.expire_allocation(origin),
            _ => {}
        }
    }
}
