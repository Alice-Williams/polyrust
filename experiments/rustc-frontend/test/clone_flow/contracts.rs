//! Whole-body evidence cannot be fabricated, erased or supplied arbitrary MIR.
#![allow(dead_code)]
#[cfg(clone_flow_private)]
fn fake(input: crate::owned_linear::cloning::CloneOwnedBody<'_>) {
    let _ = crate::owned_linear::cloning::CloneOwnedBody { ..input };
}
#[cfg(clone_flow_erased)]
fn erased(input: crate::owned_linear::cloning::CloneOwnedBody<'_>) {
    fn linear(_: crate::owned_linear::LinearOwnedBody<'_>) {}
    linear(input);
}
#[cfg(clone_flow_raw)]
fn raw<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    owner: rustc_hir::def_id::LocalDefId,
    body: &rustc_middle::mir::Body<'tcx>,
) {
    let _ = crate::owned_linear::cloning::CloneOwnedBody::read(tcx, owner, body);
}
