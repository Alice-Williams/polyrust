//! Live automatic activations and must-initialized representations.
use super::root_cells::RootCell;
use super::values::Cell;
use crate::ast::{CRegistry, CScopeRef};
use crate::ownership::{
    CSafetyError as E,
    paths::{Key, Root},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct State {
    pub(super) roots: BTreeMap<Root, RootCell>,
    pub(super) allocations: super::allocations::Allocations,
}
impl State {
    pub(super) fn live(&self, path: &Key) -> Result<(), E> {
        if let Root::Allocation(origin, _) = path.root()
            && self.allocations.nonnull(origin)? != Some(true)
        {
            return Err(E::ExpiredStorage);
        }
        self.roots
            .contains_key(path.root())
            .then_some(())
            .ok_or(E::ExpiredStorage)
    }
    pub(super) fn read(&self, path: &Key, registry: &CRegistry) -> Result<Cell, E> {
        self.live(path)?;
        self.roots[path.root()].read(path, registry)
    }
    pub(super) fn write(
        &mut self,
        path: &Key,
        value: &Cell,
        registry: &CRegistry,
    ) -> Result<(), E> {
        self.live(path)?;
        self.invalidate_indices(path.root());
        if matches!(path.root(), Root::Global(_)) && value.contains_allocation() {
            return Err(E::UnprovedAllocation);
        }
        if matches!(path.root(), Root::Global(_))
            && value.automatic_address(&path.ty(), registry)?
        {
            return Err(E::AutomaticAddressEscape);
        }
        self.roots
            .get_mut(path.root())
            .ok_or(E::ExpiredStorage)?
            .write(path, value, registry)
    }
    pub(super) fn expire(&mut self, root: &Root) {
        self.invalidate_indices(root);
        if let Root::Local(local) = root {
            self.allocations.expire_count_local(local);
        }
        self.roots.remove(root);
        for cell in self.roots.values_mut() {
            cell.each_mut(|cell| cell.expire(root));
        }
    }
    fn invalidate_indices(&mut self, root: &Root) {
        for cell in self.roots.values_mut() {
            cell.invalidate_index(root);
        }
    }
    pub(super) fn leave(&mut self, scope: &CScopeRef) {
        self.allocations.leave_count_scope(scope);
        let expired: Vec<_> = self
            .roots
            .keys()
            .filter(|root| root.leaves(scope))
            .cloned()
            .collect();
        for root in expired {
            self.expire(&root);
        }
    }
    pub(super) fn join(&self, other: &Self, registry: &CRegistry) -> Result<Self, E> {
        let mut roots = BTreeMap::new();
        for (root, left) in &self.roots {
            if let Some(right) = other.roots.get(root) {
                roots.insert(root.clone(), left.join(right, &root.shape(), registry)?);
            }
        }
        Ok(Self {
            roots,
            allocations: self.allocations.join(&other.allocations),
        })
    }
    pub(super) fn forget_values(&mut self) {
        for value in self.roots.values_mut() {
            value.forget();
        }
    }
}
