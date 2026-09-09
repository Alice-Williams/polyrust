//! Requested local roles become evidence only through actual product transfers.
mod actions;
mod claims;
#[cfg(test)]
#[path = "../../tests/owner_lattice.rs"]
mod tests;
mod transactions;

use super::state::State;
use crate::ast::{CLocalRef, CScopeRef};
use crate::ownership::{CSafetyError as E, paths::Root};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Slot {
    Uninitialized,
    Empty,
    Live(Box<Root>),
    Moved,
    Dropped,
    Unproved,
}
impl Slot {
    fn empty(&self) -> bool {
        matches!(self, Self::Empty | Self::Moved | Self::Dropped)
    }
    fn settled(&self) -> bool {
        self.empty() || matches!(self, Self::Uninitialized)
    }
    fn join(&self, other: &Self) -> Self {
        if self == other {
            self.clone()
        } else if self.empty() && other.empty() {
            Self::Empty
        } else {
            Self::Unproved
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Transaction {
    Move {
        source: CLocalRef,
        destination: Box<CLocalRef>,
        root: Box<Root>,
    },
    Drop {
        source: CLocalRef,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Owners {
    slots: BTreeMap<CLocalRef, Slot>,
    pending: Option<Transaction>,
    poisoned: bool,
}

pub(super) struct Plan {
    owners: Owners,
    transferred: Option<(Root, CLocalRef)>,
}
impl Plan {
    pub(super) fn apply(
        self,
        memory: &mut State,
        registry: &crate::ast::CRegistry,
    ) -> Result<(), E> {
        if let Some((root, destination)) = self.transferred {
            memory.transfer_owner(&root, &destination)?;
        }
        self.owners.check_live(memory, registry)?;
        memory.owners = self.owners;
        Ok(())
    }
}

impl Owners {
    pub(super) fn fail(&mut self) {
        // Never erase a pending transaction or resource on provisional failure.
        if !self.slots.is_empty() || self.pending.is_some() {
            self.poisoned = true;
        }
    }
    pub(super) fn finish(&self) -> Result<(), E> {
        if self.poisoned || self.pending.is_some() || self.slots.values().any(|s| !s.settled()) {
            Err(E::UnprovedOwnership)
        } else {
            Ok(())
        }
    }
    pub(super) fn leave(&mut self, scope: &CScopeRef) -> Result<(), E> {
        if self.poisoned
            || self.pending.is_some()
            || self
                .slots
                .iter()
                .any(|(local, slot)| local.scope() == scope && !slot.settled())
        {
            return Err(E::UnprovedOwnership);
        }
        self.slots.retain(|local, _| local.scope() != scope);
        Ok(())
    }
    pub(super) fn join(&self, other: &Self) -> Self {
        let mut slots = BTreeMap::new();
        for local in self.slots.keys().chain(other.slots.keys()) {
            let slot = match (self.slots.get(local), other.slots.get(local)) {
                (Some(left), Some(right)) => left.join(right),
                _ => Slot::Unproved,
            };
            slots.insert(local.clone(), slot);
        }
        Self {
            slots,
            pending: if self.pending == other.pending {
                self.pending.clone()
            } else {
                None
            },
            poisoned: self.poisoned || other.poisoned || self.pending != other.pending,
        }
    }
}
