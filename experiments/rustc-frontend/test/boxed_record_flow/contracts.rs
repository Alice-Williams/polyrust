//! Certificates remain private and distinct from scalar-Box body evidence.
#[cfg(boxed_flow_private_body)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::boxed_records::BoxedRecordBody<'_>) {
    let _ = crate::owned_linear::multiple::boxed_records::BoxedRecordBody { ..proof };
}
#[cfg(boxed_flow_private_field)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::multiple::boxed_records::FieldProducer<'_>) {
    let _ = crate::owned_linear::multiple::boxed_records::FieldProducer { ..proof };
}
#[cfg(boxed_flow_erased)]
fn fake(proof: crate::owned_linear::multiple::boxed_records::BoxedRecordBody<'_>) {
    let _: crate::owned_linear::LinearOwnedBody<'_> = proof;
}
#[cfg(boxed_flow_raw)]
fn fake<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    owner: rustc_hir::def_id::LocalDefId,
    body: &rustc_middle::mir::Body<'tcx>,
) {
    let _ = crate::owned_linear::multiple::boxed_records::BoxedRecordBody::read(tcx, owner, body);
}
