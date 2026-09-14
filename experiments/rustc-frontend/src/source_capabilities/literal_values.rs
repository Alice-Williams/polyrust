//! Compiler-owned input for LiteralValues; no target representation.
use super::Capability;
use rustc_hir as hir;

pub(crate) struct LiteralValues;
pub(crate) struct LiteralInput<'tcx>(pub(crate) &'tcx hir::Expr<'tcx>);

impl Capability for LiteralValues {
    type Input<'tcx> = LiteralInput<'tcx>;
}
