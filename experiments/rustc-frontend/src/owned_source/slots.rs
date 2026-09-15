//! Consuming registration with the shared executable capability traits.
use super::{
    OwnedBoxConstruction, boxed_record::OwnedScalarRecordBoxConstruction,
    record::OwnedRecordConstruction,
};
use crate::source_capabilities::{Mapping, Supports};

pub(crate) struct Missing;
pub(crate) struct Builder<M = Missing, R = Missing, P = Missing>(M, R, P);
pub(crate) struct Bindings<M, R = Missing, P = Missing>(M, R, P);

impl Builder {
    pub(crate) fn new() -> Self {
        Self(Missing, Missing, Missing)
    }
}
impl<R, P> Builder<Missing, R, P> {
    pub(crate) fn construction<M: Mapping<Capability = OwnedBoxConstruction>>(
        self,
        mapping: M,
    ) -> Builder<M, R, P> {
        Builder(mapping, self.1, self.2)
    }
}
impl<M, P> Builder<M, Missing, P> {
    // Record registration is optional for Box-only proof consumers.
    #[allow(dead_code)]
    pub(crate) fn record_construction<R: Mapping<Capability = OwnedRecordConstruction>>(
        self,
        mapping: R,
    ) -> Builder<M, R, P> {
        Builder(self.0, mapping, self.2)
    }
}
impl<M, R> Builder<M, R, Missing> {
    #[allow(dead_code)]
    pub(crate) fn scalar_record_box<P: Mapping<Capability = OwnedScalarRecordBoxConstruction>>(
        self,
        mapping: P,
    ) -> Builder<M, R, P> {
        Builder(self.0, self.1, mapping)
    }
}
impl<M: Mapping<Capability = OwnedBoxConstruction>, R, P> Builder<M, R, P> {
    pub(crate) fn build(self) -> Bindings<M, R, P> {
        Bindings(self.0, self.1, self.2)
    }
}
impl<M: Mapping<Capability = OwnedBoxConstruction>, R, P> Supports<OwnedBoxConstruction>
    for Bindings<M, R, P>
{
    type Mapping = M;
    fn mapping(&self) -> M {
        self.0
    }
}
impl<M, R: Mapping<Capability = OwnedRecordConstruction>, P> Supports<OwnedRecordConstruction>
    for Bindings<M, R, P>
{
    type Mapping = R;
    fn mapping(&self) -> R {
        self.1
    }
}
impl<M, R, P: Mapping<Capability = OwnedScalarRecordBoxConstruction>>
    Supports<OwnedScalarRecordBoxConstruction> for Bindings<M, R, P>
{
    type Mapping = P;
    fn mapping(&self) -> P {
        self.2
    }
}
