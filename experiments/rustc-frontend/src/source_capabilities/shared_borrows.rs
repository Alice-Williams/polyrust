//! Compiler-owned input for SharedBorrows; no target representation.
use super::Capability;
use rustc_hir as hir;

pub(crate) struct SharedBorrows;
pub(crate) struct BorrowInput<'tcx>(pub(crate) &'tcx hir::Expr<'tcx>);

impl Capability for SharedBorrows {
    type Input<'tcx> = BorrowInput<'tcx>;
}
