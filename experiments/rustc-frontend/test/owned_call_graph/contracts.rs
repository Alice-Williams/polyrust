//! Safe consumers cannot synthesize or erase graph/body evidence.
#[cfg(call_graph_private_graph)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::calls::OwnedCallGraph<'_>) {
    let _ = crate::owned_linear::calls::OwnedCallGraph { ..proof };
}
#[cfg(call_graph_private_body)]
#[allow(dead_code)]
fn fake(proof: crate::owned_linear::calls::OwnedCallBody<'_>) {
    let _ = crate::owned_linear::calls::OwnedCallBody { ..proof };
}
#[cfg(call_graph_erased)]
fn fake(proof: crate::owned_linear::calls::OwnedCallGraph<'_>) {
    let _: crate::owned_source::local_call::LocalCallInput<'_> = proof;
}
#[cfg(call_graph_raw)]
fn fake<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    owner: rustc_hir::def_id::LocalDefId,
    body: &rustc_middle::mir::Body<'tcx>,
) {
    let _ = crate::owned_linear::calls::OwnedCallGraph::read(tcx, owner, body);
}
#[cfg(call_graph_assembly)]
fn fake<'tcx>(
    entry: crate::owned_linear::calls::OwnedCallBody<'tcx>,
    leaf: crate::owned_linear::calls::OwnedCallBody<'tcx>,
) {
    let _ = crate::owned_linear::calls::OwnedCallGraph::assemble(entry, leaf);
}
