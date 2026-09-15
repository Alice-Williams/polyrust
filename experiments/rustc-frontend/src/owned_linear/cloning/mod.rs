//! Query-only original/clone body evidence; target heap generation is separate.
mod calls;
mod loans;
#[cfg(clone_flow_proof)]
#[path = "../../../test/clone_flow/mutations/mod.rs"]
pub(crate) mod mutations;
mod relations;
mod source;
use super::{Result, SourceExit};
use crate::owned_source::{BoxConstructionInput, cloning::BoxCloneInput};
pub(crate) use loans::Loan;
use rustc_hir::{HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Owner {
    Original,
    Cloned,
}
impl Owner {
    fn index(self) -> usize {
        match self {
            Self::Original => 0,
            Self::Cloned => 1,
        }
    }
}
pub(crate) struct CloneOwnedBody<'tcx> {
    plan: source::Plan<'tcx>,
    matched: relations::Matched<'tcx>,
}
impl<'tcx> CloneOwnedBody<'tcx> {
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        let plan = source::read(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let matched = relations::validate(tcx, &plan, &body)?;
        Ok(Self { plan, matched })
    }
    pub(crate) fn owner(&self) -> LocalDefId {
        self.plan.frame.owner
    }
    pub(crate) fn parameter(&self) -> (HirId, mir::Local) {
        (self.plan.frame.parameter, self.matched.parameter)
    }
    /// Source declaration order: first Original is construction, first Cloned is
    /// cloning, and each later occurrence of a tag moves its previous binding.
    /// Locations identify the exact call terminator or whole-owner assignment.
    pub(crate) fn bindings(&self) -> &[(Owner, HirId, mir::Local, mir::Location)] {
        &self.matched.bindings
    }
    pub(crate) fn scope(&self) -> HirId {
        self.plan.frame.scopes.read_scope()
    }
    pub(crate) fn exit(&self) -> SourceExit<'tcx> {
        self.plan.frame.scopes.exit()
    }
    pub(crate) fn returning(&self) -> mir::Location {
        self.matched.returning
    }
    pub(crate) fn loan(&self) -> Loan<'tcx> {
        self.matched.loan
    }
    pub(crate) fn read_owner(&self) -> Owner {
        self.plan.read
    }
    pub(crate) fn scalar_read(&self) -> mir::Location {
        self.matched.read
    }
    pub(crate) fn drops(&self) -> &[(Owner, mir::Place<'tcx>, mir::Location); 2] {
        &self.matched.drops
    }
    pub(crate) fn into_inputs(self) -> (BoxConstructionInput<'tcx>, BoxCloneInput<'tcx>) {
        (self.plan.construction, self.plan.cloning)
    }
}
