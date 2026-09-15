//! Private evidence cannot be fabricated, erased, or built from arbitrary MIR.
#[cfg(fields_private_body)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::records::RecordOwnedBody<'_>) {
    let _ = crate::owned_linear::multiple::records::RecordOwnedBody { ..proof };
}
#[cfg(fields_private_chain)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::records::ChainEvidence<'_>) {
    let _ = crate::owned_linear::multiple::records::ChainEvidence { ..proof };
}
#[cfg(fields_erased)]
fn fake(proof: crate::owned_linear::multiple::records::RecordOwnedBody<'_>) {
    let _: crate::owned_linear::multiple::MultipleOwnedBody<'_> = proof;
}
#[cfg(fields_raw)]
fn fake<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    owner: rustc_hir::def_id::LocalDefId,
    body: &rustc_middle::mir::Body<'tcx>,
) {
    let _ = crate::owned_linear::multiple::records::RecordOwnedBody::read(tcx, owner, body);
}
