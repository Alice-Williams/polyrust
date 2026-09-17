//! Compiler-owned input for LexicalControl; no target representation.
use super::Capability;
use rustc_hir as hir;

pub(crate) struct LexicalControl;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControlCompletion {
    Return,
    Effect,
}
pub(crate) struct ControlInput<'tcx> {
    pub(crate) expression: &'tcx hir::Expr<'tcx>,
    pub(crate) parent: Option<hir::HirId>,
    pub(crate) completion: ControlCompletion,
}
impl Capability for LexicalControl {
    type Input<'tcx> = ControlInput<'tcx>;
}
