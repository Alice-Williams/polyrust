//! Early arm plus root-scope continuation, never a fabricated else block.
use super::super::Result;
use super::guarded::{self, GuardEvidence, GuardedPath};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{mir, ty::TyCtxt};

#[cfg(owned_early_proof)]
#[path = "../../../test/owned_early/mutations.rs"]
pub(crate) mod mutations;

pub(crate) struct EarlyOwnedBody<'tcx> {
    guard: GuardEvidence<'tcx>,
    paths: [GuardedPath<'tcx>; 2],
}
impl<'tcx> EarlyOwnedBody<'tcx> {
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        guarded::source::read_early(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        Self::from_body(tcx, owner, &body)
    }
    fn from_body(tcx: TyCtxt<'tcx>, owner: LocalDefId, body: &mir::Body<'tcx>) -> Result<Self> {
        let shape = guarded::source::read_early(tcx, owner)?;
        let (guard, paths) = guarded::certify(tcx, owner, shape, body)?;
        Ok(Self { guard, paths })
    }
    pub(crate) fn guard(&self) -> &GuardEvidence<'tcx> {
        &self.guard
    }
    pub(crate) fn paths(&self) -> &[GuardedPath<'tcx>; 2] {
        &self.paths
    }
    pub(crate) fn into_parts(self) -> (GuardEvidence<'tcx>, [GuardedPath<'tcx>; 2]) {
        (self.guard, self.paths)
    }
}
