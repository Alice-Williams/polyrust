//! The sixth slot preserves every historical mapping in either registration order.
use crate::{
    owned_source::{
        Builder, OwnedBoxConstruction, boxed_record::OwnedScalarRecordBoxConstruction,
        cloning::OwnedBoxClone, local_call::OwnedLocalCall,
        nested_record::OwnedNestedRecordConstruction, record::OwnedRecordConstruction,
    },
    source_capabilities::{Capability, Mapping, Supports},
};
use std::marker::PhantomData;
struct Sentinel<C>(u8, PhantomData<fn() -> C>);
impl<C> Clone for Sentinel<C> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<C> Copy for Sentinel<C> {}
impl<C: Capability> Mapping for Sentinel<C> {
    type Capability = C;
    type Context<'tcx> = ();
    type Output = u8;
    fn lower<'tcx>(&self, _: &mut (), _: C::Input<'tcx>) -> Result<u8, String> {
        Ok(self.0)
    }
}
fn sentinel<C>(value: u8) -> Sentinel<C> {
    Sentinel(value, PhantomData)
}
fn check_slots(
    bindings: impl Supports<OwnedBoxConstruction, Mapping = Sentinel<OwnedBoxConstruction>>
    + Supports<OwnedRecordConstruction, Mapping = Sentinel<OwnedRecordConstruction>>
    + Supports<
        OwnedScalarRecordBoxConstruction,
        Mapping = Sentinel<OwnedScalarRecordBoxConstruction>,
    > + Supports<OwnedLocalCall, Mapping = Sentinel<OwnedLocalCall>>
    + Supports<OwnedBoxClone, Mapping = Sentinel<OwnedBoxClone>>
    + Supports<
        OwnedNestedRecordConstruction,
        Mapping = Sentinel<OwnedNestedRecordConstruction>,
    >,
) {
    assert_eq!(Supports::<OwnedBoxConstruction>::mapping(&bindings).0, 1);
    assert_eq!(Supports::<OwnedRecordConstruction>::mapping(&bindings).0, 2);
    assert_eq!(
        Supports::<OwnedScalarRecordBoxConstruction>::mapping(&bindings).0,
        3
    );
    assert_eq!(Supports::<OwnedLocalCall>::mapping(&bindings).0, 4);
    assert_eq!(Supports::<OwnedBoxClone>::mapping(&bindings).0, 5);
    assert_eq!(
        Supports::<OwnedNestedRecordConstruction>::mapping(&bindings).0,
        6
    );
}
pub(super) fn check() {
    check_slots(
        Builder::new()
            .construction(sentinel(1))
            .record_construction(sentinel(2))
            .scalar_record_box(sentinel(3))
            .local_call(sentinel(4))
            .box_clone(sentinel(5))
            .nested_record(sentinel(6))
            .build(),
    );
    check_slots(
        Builder::new()
            .nested_record(sentinel(6))
            .box_clone(sentinel(5))
            .local_call(sentinel(4))
            .scalar_record_box(sentinel(3))
            .record_construction(sentinel(2))
            .construction(sentinel(1))
            .build(),
    );
}
