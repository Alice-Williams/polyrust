//! Exact compile-negative contracts for complete nested ownership evidence.
#![allow(dead_code)]
#[cfg(nested_body_private)]
fn private(input: crate::owned_linear::nested::NestedOwnedBody<'_>) {
    let _ = crate::owned_linear::nested::NestedOwnedBody { ..input };
}
#[cfg(nested_body_private_path)]
fn private_path(input: crate::owned_linear::nested::path::SourcePlace<'_>) {
    let _ = crate::owned_linear::nested::path::SourcePlace { ..input };
}
#[cfg(nested_body_private_event)]
fn private_event(input: crate::owned_linear::nested::events::Movement<'_>) {
    let _ = crate::owned_linear::nested::events::Movement { ..input };
}
#[cfg(nested_body_erased)]
fn erased(input: crate::owned_source::nested_record::NestedRecordConstructionInput<'_>) {
    fn require(_: crate::owned_linear::nested::NestedOwnedBody<'_>) {}
    require(input);
}
#[cfg(nested_body_raw)]
fn raw<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    owner: rustc_hir::def_id::LocalDefId,
    body: &rustc_middle::mir::Body<'tcx>,
) {
    let _ = crate::owned_linear::nested::NestedOwnedBody::read(tcx, owner, body);
}
