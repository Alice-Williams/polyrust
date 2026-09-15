//! Query-only two-function evidence; no target heap admission follows.
mod evidence;
mod frame;
mod moves;
#[cfg(owned_call_graph_proof)]
#[path = "../../../test/owned_call_graph/mutations.rs"]
pub(crate) mod mutations;
mod relations;
mod source;
pub(crate) use super::exits::Exit as SourceExit;
use super::{LinearError as Error, Result};
use crate::owned_source::BoxConstructionInput;
use crate::owned_source::local_call::{CallRole, LocalCallInput};
pub(crate) use evidence::{BodyEnding, BodyStep};
use rustc_hir::{HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};

pub(crate) struct OwnedCallBody<'tcx> {
    plan: source::Plan<'tcx>,
    matched: relations::Matched,
}
pub(crate) enum CheckedInput<'tcx> {
    Construction(BoxConstructionInput<'tcx>),
    LocalCall(LocalCallInput<'tcx>),
}
impl<'tcx> OwnedCallBody<'tcx> {
    fn from_plan(tcx: TyCtxt<'tcx>, plan: source::Plan<'tcx>) -> Result<Self> {
        let body = tcx
            .mir_drops_elaborated_and_const_checked(plan.frame.owner)
            .borrow();
        let matched = relations::validate(tcx, &plan, &body)?;
        Ok(Self { plan, matched })
    }
    pub(crate) fn owner(&self) -> LocalDefId {
        self.plan.frame.owner
    }
    pub(crate) fn parameter(&self) -> (HirId, mir::Local) {
        (self.plan.frame.parameter, self.matched.parameter)
    }
    pub(crate) fn bindings(&self) -> &[(HirId, mir::Local)] {
        &self.matched.bindings
    }
    pub(crate) fn exit(&self) -> SourceExit<'tcx> {
        self.plan.frame.scopes.exit()
    }
    pub(crate) fn returning(&self) -> mir::Location {
        self.matched.returning
    }
    pub(crate) fn steps(&self) -> &[BodyStep] {
        &self.matched.steps
    }
    pub(crate) fn ending(&self) -> &BodyEnding {
        &self.matched.ending
    }
    pub(crate) fn into_inputs(self) -> Vec<CheckedInput<'tcx>> {
        self.plan
            .operations
            .into_iter()
            .filter_map(|op| match op {
                source::Operation::Allocate { input, .. } => {
                    Some(CheckedInput::Construction(input))
                }
                source::Operation::Call { input, .. } => Some(CheckedInput::LocalCall(input)),
                source::Operation::Move(_) => None,
            })
            .collect()
    }
}
pub(crate) struct OwnedCallGraph<'tcx> {
    entry: OwnedCallBody<'tcx>,
    leaf: OwnedCallBody<'tcx>,
}
impl<'tcx> OwnedCallGraph<'tcx> {
    pub(crate) fn read(tcx: TyCtxt<'tcx>, entry: LocalDefId) -> Result<Self> {
        let entry_plan = source::entry(tcx, entry)?;
        let call = entry_plan.call()?;
        if call.callee() == entry {
            return Err(Error::Call);
        }
        let leaf = OwnedCallBody::from_plan(tcx, source::leaf(tcx, call)?)?;
        let entry = OwnedCallBody::from_plan(tcx, entry_plan)?;
        Self::assemble(entry, leaf)
    }
    fn assemble(entry: OwnedCallBody<'tcx>, leaf: OwnedCallBody<'tcx>) -> Result<Self> {
        let call = entry.plan.call()?;
        if call.owner() != entry.owner()
            || call.callee() != leaf.owner()
            || entry.owner() == leaf.owner()
            || call.signature() != leaf.plan.frame.signature
            || call.box_ty() != leaf.plan.box_ty
            || leaf.plan.call().is_ok()
        {
            return Err(Error::Call);
        }
        Ok(Self { entry, leaf })
    }
    pub(crate) fn entry(&self) -> &OwnedCallBody<'tcx> {
        &self.entry
    }
    pub(crate) fn leaf(&self) -> &OwnedCallBody<'tcx> {
        &self.leaf
    }
    pub(crate) fn call(&self) -> &LocalCallInput<'tcx> {
        self.entry.plan.call().unwrap()
    }
    pub(crate) fn role(&self) -> CallRole {
        self.call().role()
    }
    pub(crate) fn into_bodies(self) -> [OwnedCallBody<'tcx>; 2] {
        [self.entry, self.leaf]
    }
}
