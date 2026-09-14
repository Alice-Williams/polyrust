//! Compiler-owned input for LexicalControl; no target representation.
use super::Capability;
use rustc_hir as hir;

pub(crate) struct LexicalControl;
pub(crate) struct ControlInput<'tcx> {
    pub(crate) expression: &'tcx hir::Expr<'tcx>,
    pub(crate) parent: Option<hir::HirId>,
}
impl Capability for LexicalControl {
    type Input<'tcx> = ControlInput<'tcx>;
}
