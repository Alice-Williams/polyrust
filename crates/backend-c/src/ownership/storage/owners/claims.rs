//! Complete fixed leaf claims retain the actual allocation and its numeric history.
use super::super::{state::State, values::Pointer};
use super::{Owners, Slot};
use crate::ast::{CAggregateRef, CObjectType, CObjectTypeKind, CRegistry};
use crate::ownership::{
    CSafetyError as E,
    paths::{Key, Root, Shape},
};
use std::collections::BTreeSet;

impl Owners {
    pub(super) fn check_live(&self, memory: &State, registry: &CRegistry) -> Result<(), E> {
        for (local, slot) in &self.slots {
            let Slot::Live(root) = slot else { continue };
            // A verified release has deliberately retired this root. Only its
            // exact pending reset may follow, not another use or control choice.
            if matches!(&self.pending, Some(super::Transaction::Drop { source }) if source == local)
            {
                continue;
            }
            let path = Key::from_root(root.as_ref().clone());
            let pointer = memory.read(&Key::local(local), registry)?.pointer()?;
            if pointer != Pointer::Target(Box::new(path.clone())) {
                return Err(E::UnprovedOwnership);
            }
            memory
                .read(&path, registry)?
                .complete(&path.ty(), registry)?;
        }
        Ok(())
    }

    pub(super) fn claim(
        &self,
        pointer: Pointer,
        memory: &State,
        registry: &CRegistry,
    ) -> Result<Root, E> {
        let Pointer::Target(path) = pointer else {
            return Err(E::UnprovedOwnership);
        };
        let Root::Allocation(_, Shape::Object(ty)) = path.root() else {
            return Err(E::UnprovedOwnership);
        };
        if !path.whole_root()
            || self
                .slots
                .values()
                .any(|slot| matches!(slot, Slot::Live(root) if root.as_ref() == path.root()))
        {
            return Err(E::UnprovedOwnership);
        }
        leaf_type(ty, registry)?;
        memory.read(&path, registry)?.complete(ty, registry)?;
        Ok(path.root().clone())
    }
}

fn leaf_type(ty: &CObjectType, registry: &CRegistry) -> Result<(), E> {
    let mut pending = vec![ty.clone()];
    let mut seen = BTreeSet::new();
    while let Some(ty) = pending.pop() {
        let owner = match ty.canonical().kind() {
            CObjectTypeKind::Pointer(_) => return Err(E::UnprovedOwnership),
            CObjectTypeKind::Array { element, .. } => {
                pending.push(element.as_ref().clone());
                None
            }
            CObjectTypeKind::Struct(owner) => Some(CAggregateRef::Struct(owner.clone())),
            CObjectTypeKind::Union(owner) => Some(CAggregateRef::Union(owner.clone())),
            _ => None,
        };
        if let Some(owner) = owner
            && seen.insert(owner.clone())
        {
            pending.extend(
                registry
                    .members(&owner)?
                    .ok_or(E::IncompleteLayout)?
                    .iter()
                    .map(|m| m.ty().clone()),
            );
        }
    }
    Ok(())
}

impl State {
    pub(super) fn transfer_owner(
        &mut self,
        root: &Root,
        destination: &crate::ast::CLocalRef,
    ) -> Result<(), E> {
        let Root::Allocation(origin, Shape::Object(_)) = root else {
            return Err(E::UnprovedOwnership);
        };
        let value = self.roots.get(root).cloned().ok_or(E::ExpiredStorage)?;
        // This retires addresses, NOT the physical allocation or numeric history.
        // Pointer-free payloads cannot contain aliases restored by this copy.
        self.expire_allocation(origin);
        self.roots.insert(root.clone(), value);
        self.roots.insert(
            Root::Local(destination.clone()),
            super::super::root_cells::RootCell::object(super::super::values::Cell::Pointer(
                Pointer::Target(Box::new(Key::from_root(root.clone()))),
            )),
        );
        Ok(())
    }
}
