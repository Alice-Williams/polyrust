//! Compiler-owned input for FunctionSignatures; no target representation.
use super::Capability;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;

pub(crate) struct FunctionSignatures;
pub(crate) struct FunctionInput<'tcx> {
    pub(crate) tcx: TyCtxt<'tcx>,
    pub(crate) function: DefId,
}
impl Capability for FunctionSignatures {
    type Input<'tcx> = FunctionInput<'tcx>;
}
