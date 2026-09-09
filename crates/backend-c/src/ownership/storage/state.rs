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
    pub(super) prefixes: BTreeMap<Root, super::prefixes::Prefixes>,
    pub(super) allocations: super::allocations::Allocations,
    pub(super) owners: super::owners::Owners,
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
        if let Some(prefixes) = self.prefixes.get_mut(path.root())
            && let crate::ownership::paths::Shape::Elements { element, .. } = path.root().shape()
        {
            prefixes.write(path, value, &element, registry)?;
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
        self.prefixes.remove(root);
        for prefixes in self.prefixes.values_mut() {
            prefixes.each_mut(|cell| cell.expire(root));
        }
        for cell in self.roots.values_mut() {
            cell.each_mut(|cell| cell.expire(root));
        }
    }
    fn invalidate_indices(&mut self, root: &Root) {
        for prefixes in self.prefixes.values_mut() {
            prefixes.invalidate_index(root);
        }
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
        let mut prefixes = BTreeMap::new();
        for (root, left) in &self.prefixes {
            if let Some(right) = other.prefixes.get(root)
                && roots.contains_key(root)
                && let crate::ownership::paths::Shape::Elements { element, .. } = root.shape()
            {
                prefixes.insert(root.clone(), left.join(right, &element, registry)?);
            }
        }
        Ok(Self {
            roots,
            prefixes,
            allocations: self.allocations.join(&other.allocations),
            owners: self.owners.join(&other.owners),
        })
    }
    pub(super) fn forget_values(&mut self) {
        self.prefixes.clear();
        for value in self.roots.values_mut() {
            value.forget();
        }
    }
}
