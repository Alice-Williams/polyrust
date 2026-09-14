//! Compiler-owned input for EntrySignatures; no target representation.
use super::Capability;
use rustc_hir::def_id::LocalDefId;
use rustc_middle::ty::TyCtxt;

pub(crate) struct EntrySignatures;
pub(crate) struct EntryInput<'tcx> {
    pub(crate) tcx: TyCtxt<'tcx>,
    pub(crate) root: LocalDefId,
}
impl Capability for EntrySignatures {
    type Input<'tcx> = EntryInput<'tcx>;
}
