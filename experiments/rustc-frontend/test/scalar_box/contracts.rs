//! Compile-negative signatures fail only for their intended type boundary.
#![allow(dead_code)]
#[cfg(any(
    scalar_box_missing,
    scalar_box_missing_base,
    scalar_box_duplicate,
    scalar_box_wrong_capability
))]
use crate::owned_source::Builder;
#[cfg(scalar_box_missing)]
use crate::owned_source::boxed_record::OwnedScalarRecordBoxConstruction;
#[cfg(scalar_box_missing)]
use crate::source_capabilities::Supports;
#[cfg(scalar_box_missing)]
fn missing() {
    fn needs(_: impl Supports<OwnedScalarRecordBoxConstruction>) {}
    needs(
        Builder::new()
            .construction(super::previous::BoxMapping)
            .build(),
    );
}
#[cfg(scalar_box_missing_base)]
fn missing_base() {
    let _ = Builder::new()
        .scalar_record_box(super::mapping::Observe)
        .build();
}
#[cfg(scalar_box_duplicate)]
fn duplicate() {
    let _ = Builder::new()
        .scalar_record_box(super::mapping::Observe)
        .scalar_record_box(super::mapping::Observe);
}
#[cfg(scalar_box_wrong_capability)]
fn wrong_capability() {
    let _ = Builder::new().scalar_record_box(super::previous::BoxMapping);
}
#[cfg(scalar_box_wrong_input)]
fn wrong_input<'tcx>(
    context: &mut super::mapping::Context<'tcx>,
    input: &'tcx rustc_hir::Expr<'tcx>,
) {
    use crate::source_capabilities::Mapping;
    let _ = super::mapping::Observe.lower(context, input);
}
#[cfg(scalar_box_private_input)]
fn fake(input: crate::owned_source::boxed_record::ScalarRecordBoxInput<'_>) {
    let _ = crate::owned_source::boxed_record::ScalarRecordBoxInput { ..input };
}
#[cfg(scalar_box_private_payload)]
fn fake(input: crate::owned_source::scalar_record::ScalarRecord<'_>) {
    let _ = crate::owned_source::scalar_record::ScalarRecord { ..input };
}
#[cfg(scalar_box_private_field)]
fn fake(input: crate::owned_source::scalar_record::ScalarField<'_>) {
    let _ = crate::owned_source::scalar_record::ScalarField { ..input };
}
#[cfg(any(scalar_box_wrong_context, scalar_box_wrong_output))]
mod wrong {
    use crate::{
        owned_source::boxed_record::{OwnedScalarRecordBoxConstruction, ScalarRecordBoxInput},
        source_capabilities::Mapping,
    };
    #[derive(Clone, Copy)]
    struct Wrong;
    impl Mapping for Wrong {
        type Capability = OwnedScalarRecordBoxConstruction;
        #[cfg(scalar_box_wrong_context)]
        type Context<'tcx> = ();
        #[cfg(scalar_box_wrong_output)]
        type Context<'tcx> = super::super::mapping::Context<'tcx>;
        #[cfg(scalar_box_wrong_context)]
        type Output = rustc_hir::def_id::DefId;
        #[cfg(scalar_box_wrong_output)]
        type Output = ();
        fn lower<'tcx>(
            &self,
            _: &mut Self::Context<'tcx>,
            _: ScalarRecordBoxInput<'tcx>,
        ) -> Result<Self::Output, String> {
            unreachable!()
        }
    }
    fn wrong() {
        let _ = super::super::mapping::bindings(Wrong);
    }
}
