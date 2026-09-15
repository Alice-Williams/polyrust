//! Consuming registration with the shared executable capability traits.
use super::{
    OwnedBoxConstruction, boxed_record::OwnedScalarRecordBoxConstruction, cloning::OwnedBoxClone,
    local_call::OwnedLocalCall, record::OwnedRecordConstruction,
};
use crate::source_capabilities::{Mapping, Supports};

pub(crate) struct Missing;
pub(crate) struct Builder<M = Missing, R = Missing, P = Missing, F = Missing, C = Missing>(
    M,
    R,
    P,
    F,
    C,
);
pub(crate) struct Bindings<M, R = Missing, P = Missing, F = Missing, C = Missing>(M, R, P, F, C);

impl Builder {
    pub(crate) fn new() -> Self {
        Self(Missing, Missing, Missing, Missing, Missing)
    }
}
impl<R, P, F, C> Builder<Missing, R, P, F, C> {
    pub(crate) fn construction<M: Mapping<Capability = OwnedBoxConstruction>>(
        self,
        mapping: M,
    ) -> Builder<M, R, P, F, C> {
        Builder(mapping, self.1, self.2, self.3, self.4)
    }
}
impl<M, P, F, C> Builder<M, Missing, P, F, C> {
    // Record registration is optional for Box-only proof consumers.
    #[allow(dead_code)]
    pub(crate) fn record_construction<R: Mapping<Capability = OwnedRecordConstruction>>(
        self,
        mapping: R,
    ) -> Builder<M, R, P, F, C> {
        Builder(self.0, mapping, self.2, self.3, self.4)
    }
}
impl<M, R, F, C> Builder<M, R, Missing, F, C> {
    #[allow(dead_code)]
    pub(crate) fn scalar_record_box<P: Mapping<Capability = OwnedScalarRecordBoxConstruction>>(
        self,
        mapping: P,
    ) -> Builder<M, R, P, F, C> {
        Builder(self.0, self.1, mapping, self.3, self.4)
    }
}
impl<M, R, P, C> Builder<M, R, P, Missing, C> {
    #[allow(dead_code)]
    pub(crate) fn local_call<F: Mapping<Capability = OwnedLocalCall>>(
        self,
        mapping: F,
    ) -> Builder<M, R, P, F, C> {
        Builder(self.0, self.1, self.2, mapping, self.4)
    }
}
impl<M: Mapping<Capability = OwnedBoxConstruction>, R, P, F, C> Builder<M, R, P, F, C> {
    pub(crate) fn build(self) -> Bindings<M, R, P, F, C> {
        Bindings(self.0, self.1, self.2, self.3, self.4)
    }
}
impl<M: Mapping<Capability = OwnedBoxConstruction>, R, P, F, C> Supports<OwnedBoxConstruction>
    for Bindings<M, R, P, F, C>
{
    type Mapping = M;
    fn mapping(&self) -> M {
        self.0
    }
}
impl<M, R: Mapping<Capability = OwnedRecordConstruction>, P, F, C> Supports<OwnedRecordConstruction>
    for Bindings<M, R, P, F, C>
{
    type Mapping = R;
    fn mapping(&self) -> R {
        self.1
    }
}
impl<M, R, P: Mapping<Capability = OwnedScalarRecordBoxConstruction>, F, C>
    Supports<OwnedScalarRecordBoxConstruction> for Bindings<M, R, P, F, C>
{
    type Mapping = P;
    fn mapping(&self) -> P {
        self.2
    }
}
impl<M, R, P, F: Mapping<Capability = OwnedLocalCall>, C> Supports<OwnedLocalCall>
    for Bindings<M, R, P, F, C>
{
    type Mapping = F;
    fn mapping(&self) -> F {
        self.3
    }
}

impl<M, R, P, F> Builder<M, R, P, F, Missing> {
    #[allow(dead_code)]
    pub(crate) fn box_clone<C: Mapping<Capability = OwnedBoxClone>>(
        self,
        mapping: C,
    ) -> Builder<M, R, P, F, C> {
        Builder(self.0, self.1, self.2, self.3, mapping)
    }
}
impl<M, R, P, F, C: Mapping<Capability = OwnedBoxClone>> Supports<OwnedBoxClone>
    for Bindings<M, R, P, F, C>
{
    type Mapping = C;
    fn mapping(&self) -> C {
        self.4
    }
}
