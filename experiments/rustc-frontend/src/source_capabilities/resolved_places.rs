//! Compiler-owned input for ResolvedPlaces; no target representation.
use super::Capability;
use rustc_hir as hir;

pub(crate) struct ResolvedPlaces;
pub(crate) struct PlaceInput<'tcx>(pub(crate) &'tcx hir::Expr<'tcx>);

impl Capability for ResolvedPlaces {
    type Input<'tcx> = PlaceInput<'tcx>;
}
