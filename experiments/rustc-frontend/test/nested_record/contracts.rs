//! Exact boundary errors, never arbitrary compilation failure.
#![allow(dead_code)]
#[cfg(nested_missing)]
fn missing() {
    fn require(
        _: impl crate::source_capabilities::Supports<
            crate::owned_source::nested_record::OwnedNestedRecordConstruction,
        >,
    ) {
    }
    require(
        crate::owned_source::Builder::new()
            .construction(super::mapping::BoxMapping)
            .build(),
    );
}
#[cfg(nested_missing_base)]
fn missing_base() {
    let _ = crate::owned_source::Builder::new()
        .nested_record(super::mapping::RecordMapping)
        .build();
}
#[cfg(nested_duplicate)]
fn duplicate() {
    let _ = crate::owned_source::Builder::new()
        .nested_record(super::mapping::RecordMapping)
        .nested_record(super::mapping::RecordMapping);
}
#[cfg(nested_wrong_capability)]
fn wrong_capability() {
    let _ = crate::owned_source::Builder::new().nested_record(super::mapping::BoxMapping);
}
#[cfg(nested_wrong_input)]
fn wrong_input<'tcx>(
    context: &mut super::mapping::Context<'tcx>,
    input: &'tcx rustc_hir::Expr<'tcx>,
) {
    use crate::source_capabilities::Mapping;
    let _ = super::mapping::RecordMapping.lower(context, input);
}
#[cfg(nested_private_input)]
fn private_input(input: crate::owned_source::nested_record::NestedRecordConstructionInput<'_>) {
    let _ = crate::owned_source::nested_record::NestedRecordConstructionInput { ..input };
}
#[cfg(nested_private_field)]
fn private_field(input: crate::owned_source::nested_record::Field<'_>) {
    let _ = crate::owned_source::nested_record::Field { ..input };
}
#[cfg(any(nested_wrong_context, nested_wrong_output))]
mod wrong_signature {
    use crate::{
        owned_source::nested_record::{
            NestedRecordConstructionInput, OwnedNestedRecordConstruction,
        },
        source_capabilities::Mapping,
    };
    #[derive(Clone, Copy)]
    struct Wrong;
    impl Mapping for Wrong {
        type Capability = OwnedNestedRecordConstruction;
        #[cfg(nested_wrong_context)]
        type Context<'tcx> = ();
        #[cfg(nested_wrong_output)]
        type Context<'tcx> = super::super::mapping::Context<'tcx>;
        #[cfg(nested_wrong_context)]
        type Output = rustc_hir::def_id::DefId;
        #[cfg(nested_wrong_output)]
        type Output = ();
        fn lower<'tcx>(
            &self,
            _: &mut Self::Context<'tcx>,
            _: NestedRecordConstructionInput<'tcx>,
        ) -> Result<Self::Output, String> {
            unreachable!()
        }
    }
    fn wrong() {
        let _ = super::super::mapping::bindings(Wrong);
    }
}

#[cfg(nested_private_layout)]
fn private_layout(input: crate::owned_source::nested_record::RecordLayout<'_>) {
    let _ = crate::owned_source::nested_record::RecordLayout { ..input };
}
#[cfg(nested_private_initializer)]
fn private_initializer(input: crate::owned_source::nested_record::Initializer<'_>) {
    let _ = crate::owned_source::nested_record::Initializer { ..input };
}
