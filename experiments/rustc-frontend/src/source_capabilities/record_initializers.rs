//! Compiler-owned input for RecordInitializers; no target representation.
use super::Capability;
use rustc_hir as hir;

pub(crate) struct RecordInitializers;
pub(crate) struct RecordInput<'tcx>(pub(crate) &'tcx hir::Expr<'tcx>);
impl Capability for RecordInitializers {
    type Input<'tcx> = RecordInput<'tcx>;
}
