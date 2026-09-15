//! Consuming registration with the shared executable capability traits.
use super::{
    OwnedBoxConstruction, boxed_record::OwnedScalarRecordBoxConstruction,
    local_call::OwnedLocalCall, record::OwnedRecordConstruction,
};
use crate::source_capabilities::{Mapping, Supports};

pub(crate) struct Missing;
pub(crate) struct Builder<M = Missing, R = Missing, P = Missing, F = Missing>(M, R, P, F);
pub(crate) struct Bindings<M, R = Missing, P = Missing, F = Missing>(M, R, P, F);

impl Builder {
    pub(crate) fn new() -> Self {
        Self(Missing, Missing, Missing, Missing)
    }
}
impl<R, P, F> Builder<Missing, R, P, F> {
    pub(crate) fn construction<M: Mapping<Capability = OwnedBoxConstruction>>(
        self,
        mapping: M,
    ) -> Builder<M, R, P, F> {
        Builder(mapping, self.1, self.2, self.3)
    }
}
impl<M, P, F> Builder<M, Missing, P, F> {
    // Record registration is optional for Box-only proof consumers.
    #[allow(dead_code)]
    pub(crate) fn record_construction<R: Mapping<Capability = OwnedRecordConstruction>>(
        self,
        mapping: R,
    ) -> Builder<M, R, P, F> {
        Builder(self.0, mapping, self.2, self.3)
    }
}
impl<M, R, F> Builder<M, R, Missing, F> {
    #[allow(dead_code)]
    pub(crate) fn scalar_record_box<P: Mapping<Capability = OwnedScalarRecordBoxConstruction>>(
        self,
        mapping: P,
    ) -> Builder<M, R, P, F> {
        Builder(self.0, self.1, mapping, self.3)
    }
}
impl<M, R, P> Builder<M, R, P, Missing> {
    #[allow(dead_code)]
    pub(crate) fn local_call<F: Mapping<Capability = OwnedLocalCall>>(
        self,
        mapping: F,
    ) -> Builder<M, R, P, F> {
        Builder(self.0, self.1, self.2, mapping)
    }
}
impl<M: Mapping<Capability = OwnedBoxConstruction>, R, P, F> Builder<M, R, P, F> {
    pub(crate) fn build(self) -> Bindings<M, R, P, F> {
        Bindings(self.0, self.1, self.2, self.3)
    }
}
impl<M: Mapping<Capability = OwnedBoxConstruction>, R, P, F> Supports<OwnedBoxConstruction>
    for Bindings<M, R, P, F>
{
    type Mapping = M;
    fn mapping(&self) -> M {
        self.0
    }
}
impl<M, R: Mapping<Capability = OwnedRecordConstruction>, P, F> Supports<OwnedRecordConstruction>
    for Bindings<M, R, P, F>
{
    type Mapping = R;
    fn mapping(&self) -> R {
        self.1
    }
}
impl<M, R, P: Mapping<Capability = OwnedScalarRecordBoxConstruction>, F>
    Supports<OwnedScalarRecordBoxConstruction> for Bindings<M, R, P, F>
{
    type Mapping = P;
    fn mapping(&self) -> P {
        self.2
    }
}
impl<M, R, P, F: Mapping<Capability = OwnedLocalCall>> Supports<OwnedLocalCall>
    for Bindings<M, R, P, F>
{
    type Mapping = F;
    fn mapping(&self) -> F {
        self.3
    }
}
