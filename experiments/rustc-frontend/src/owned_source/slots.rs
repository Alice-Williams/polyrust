//! Consuming registration with the shared executable capability traits.
use super::OwnedBoxConstruction;
use crate::source_capabilities::{Mapping, Supports};

pub(crate) struct Missing;
pub(crate) struct Builder<M = Missing>(M);
pub(crate) struct Bindings<M>(M);

impl Builder {
    pub(crate) fn new() -> Self {
        Self(Missing)
    }

    pub(crate) fn construction<M: Mapping<Capability = OwnedBoxConstruction>>(
        self,
        mapping: M,
    ) -> Builder<M> {
        Builder(mapping)
    }
}

impl<M: Mapping<Capability = OwnedBoxConstruction>> Builder<M> {
    pub(crate) fn build(self) -> Bindings<M> {
        Bindings(self.0)
    }
}

impl<M: Mapping<Capability = OwnedBoxConstruction>> Supports<OwnedBoxConstruction> for Bindings<M> {
    type Mapping = M;
    fn mapping(&self) -> M {
        self.0
    }
}
