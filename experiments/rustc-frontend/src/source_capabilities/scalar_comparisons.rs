//! Compiler-owned input for ScalarComparisons; no target representation.
use super::Capability;
use rustc_hir as hir;

pub(crate) struct ScalarComparisons;
pub(crate) struct ComparisonInput<'tcx>(pub(crate) &'tcx hir::Expr<'tcx>);

impl Capability for ScalarComparisons {
    type Input<'tcx> = ComparisonInput<'tcx>;
}
