//! Live automatic activations and must-initialized representations.
use super::values::Cell;
use crate::ast::{CRegistry, CScopeRef};
use crate::ownership::{
    CSafetyError as E,
    paths::{Key, Root},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct State {
    pub(super) roots: BTreeMap<Root, Cell>,
}
impl State {
    pub(super) fn live(&self, path: &Key) -> Result<(), E> {
        self.roots
            .contains_key(path.root())
            .then_some(())
            .ok_or(E::ExpiredStorage)
    }
    pub(super) fn read(&self, path: &Key, registry: &CRegistry) -> Result<Cell, E> {
        self.live(path)?;
        self.roots[path.root()].read(&path.root().ty(), path.selectors(), registry)
    }
    pub(super) fn write(
        &mut self,
        path: &Key,
        value: &Cell,
        registry: &CRegistry,
    ) -> Result<(), E> {
        self.live(path)?;
        if matches!(path.root(), Root::Global(_))
            && value.automatic_address(&path.ty(), registry)?
        {
            return Err(E::AutomaticAddressEscape);
        }
        self.roots
            .get_mut(path.root())
            .ok_or(E::ExpiredStorage)?
            .write(&path.root().ty(), path.selectors(), value, registry)
    }
    pub(super) fn expire(&mut self, root: &Root) {
        self.roots.remove(root);
        for cell in self.roots.values_mut() {
            cell.expire(root);
        }
    }
    pub(super) fn leave(&mut self, scope: &CScopeRef) {
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
                roots.insert(root.clone(), left.join(right, &root.ty(), registry)?);
            }
        }
        Ok(Self { roots })
    }
    pub(super) fn forget_values(&mut self) {
        for value in self.roots.values_mut() {
            *value = Cell::Uninitialized;
        }
    }
}
