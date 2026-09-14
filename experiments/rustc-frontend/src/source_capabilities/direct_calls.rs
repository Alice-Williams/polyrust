//! Compiler-owned input for DirectCalls; no target representation.
use super::Capability;
use rustc_hir as hir;

pub(crate) struct DirectCalls;
pub(crate) struct CallInput<'tcx>(pub(crate) &'tcx hir::Expr<'tcx>);
impl Capability for DirectCalls {
    type Input<'tcx> = CallInput<'tcx>;
}
