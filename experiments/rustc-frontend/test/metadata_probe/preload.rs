//! Probe fixed compiler configuration that resolves unused explicit metadata.
use rustc_hir::def_id::CrateNum;
use rustc_metadata::creader::CStore;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;

pub(super) fn resolved(tcx: TyCtxt<'_>) -> Option<CrateNum> {
    CStore::from_tcx(tcx).resolved_extern_crate(Symbol::intern("renamed"))
}
