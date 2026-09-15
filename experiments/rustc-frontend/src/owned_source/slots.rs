//! Consuming registration with the shared executable capability traits.
use super::OwnedBoxConstruction;
use super::record::OwnedRecordConstruction;
use crate::source_capabilities::{Mapping, Supports};

pub(crate) struct Missing;
pub(crate) struct Builder<M = Missing, R = Missing>(M, R);
pub(crate) struct Bindings<M, R = Missing>(M, R);

impl Builder {
    pub(crate) fn new() -> Self {
        Self(Missing, Missing)
    }
}
impl<R> Builder<Missing, R> {
    pub(crate) fn construction<M: Mapping<Capability = OwnedBoxConstruction>>(
        self,
        mapping: M,
    ) -> Builder<M, R> {
        Builder(mapping, self.1)
    }
}
impl<M> Builder<M, Missing> {
    // Record registration is optional for Box-only proof consumers.
    #[allow(dead_code)]
    pub(crate) fn record_construction<R: Mapping<Capability = OwnedRecordConstruction>>(
        self,
        mapping: R,
    ) -> Builder<M, R> {
        Builder(self.0, mapping)
    }
}

impl<M: Mapping<Capability = OwnedBoxConstruction>, R> Builder<M, R> {
    pub(crate) fn build(self) -> Bindings<M, R> {
        Bindings(self.0, self.1)
    }
}

impl<M: Mapping<Capability = OwnedBoxConstruction>, R> Supports<OwnedBoxConstruction>
    for Bindings<M, R>
{
    type Mapping = M;
    fn mapping(&self) -> M {
        self.0
    }
}

impl<M, R: Mapping<Capability = OwnedRecordConstruction>> Supports<OwnedRecordConstruction>
    for Bindings<M, R>
{
    type Mapping = R;
    fn mapping(&self) -> R {
        self.1
    }
}
