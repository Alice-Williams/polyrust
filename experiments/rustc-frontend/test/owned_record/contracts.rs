//! Exact boundary errors, never arbitrary compilation failure.
#![allow(dead_code)]
#[cfg(record_missing)]
fn missing() {
    fn require(
        _: impl crate::source_capabilities::Supports<
            crate::owned_source::record::OwnedRecordConstruction,
        >,
    ) {
    }
    require(
        crate::owned_source::Builder::new()
            .construction(super::mapping::BoxMapping)
            .build(),
    );
}
#[cfg(record_missing_base)]
fn missing_base() {
    let _ = crate::owned_source::Builder::new()
        .record_construction(super::mapping::RecordMapping)
        .build();
}
#[cfg(record_duplicate)]
fn duplicate() {
    let _ = crate::owned_source::Builder::new()
        .record_construction(super::mapping::RecordMapping)
        .record_construction(super::mapping::RecordMapping);
}
#[cfg(record_wrong_capability)]
fn wrong_capability() {
    let _ = crate::owned_source::Builder::new().record_construction(super::mapping::BoxMapping);
}
#[cfg(record_wrong_input)]
fn wrong_input<'tcx>(
    context: &mut super::mapping::Context<'tcx>,
    input: &'tcx rustc_hir::Expr<'tcx>,
) {
    use crate::source_capabilities::Mapping;
    let _ = super::mapping::RecordMapping.lower(context, input);
}
#[cfg(record_private_input)]
fn private_input(input: crate::owned_source::record::RecordConstructionInput<'_>) {
    let _ = crate::owned_source::record::RecordConstructionInput { ..input };
}
#[cfg(record_private_field)]
fn private_field(input: crate::owned_source::record::RecordField<'_>) {
    let _ = crate::owned_source::record::RecordField { ..input };
}
#[cfg(any(record_wrong_context, record_wrong_output))]
mod wrong_signature {
    use crate::{
        owned_source::record::{OwnedRecordConstruction, RecordConstructionInput},
        source_capabilities::Mapping,
    };
    #[derive(Clone, Copy)]
    struct Wrong;
    impl Mapping for Wrong {
        type Capability = OwnedRecordConstruction;
        #[cfg(record_wrong_context)]
        type Context<'tcx> = ();
        #[cfg(record_wrong_output)]
        type Context<'tcx> = super::super::mapping::Context<'tcx>;
        #[cfg(record_wrong_context)]
        type Output = rustc_hir::def_id::DefId;
        #[cfg(record_wrong_output)]
        type Output = ();
        fn lower<'tcx>(
            &self,
            _: &mut Self::Context<'tcx>,
            _: RecordConstructionInput<'tcx>,
        ) -> Result<Self::Output, String> {
            unreachable!()
        }
    }
    fn wrong() {
        let _ = super::super::mapping::bindings(Wrong);
    }
}
