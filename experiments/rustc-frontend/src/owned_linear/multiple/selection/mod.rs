//! Closed source selection and complete compiler cleanup paths.
#[cfg(owned_selection_proof)]
#[path = "../../../../test/owned_selection/mutations.rs"]
pub(crate) mod mutations;
pub(super) mod source;
use super::super::{LinearError as Error, Result, flow, scopes::ScopeFacts, values};
pub(crate) use super::super::{
    exits::{Exit as SourceExit, Outcome},
    flow::flags::Decision,
};
use super::{ChainEvidence, MultipleOwnedBody, relations};
use rustc_hir::{self as hir, HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};
use std::collections::HashMap;

pub(crate) struct SelectionPath<'tcx> {
    outcome: Outcome,
    body: MultipleOwnedBody<'tcx>,
    decisions: Vec<Decision>,
    returning: mir::Location,
}
impl<'tcx> SelectionPath<'tcx> {
    pub(crate) fn outcome(&self) -> Outcome {
        self.outcome
    }
    pub(crate) fn chains(&self) -> &[ChainEvidence<'tcx>] {
        self.body.chains()
    }
    pub(crate) fn scopes(&self) -> &ScopeFacts<'tcx> {
        self.body.scopes()
    }
    pub(crate) fn exit(&self) -> SourceExit<'tcx> {
        self.body.scopes.exit()
    }
    pub(crate) fn scalar_read(&self) -> (HirId, mir::Location) {
        self.body.scalar_read()
    }
    pub(crate) fn drop_order(&self) -> &[HirId] {
        self.body.drop_order()
    }
    pub(crate) fn decisions(&self) -> &[Decision] {
        &self.decisions
    }
    pub(crate) fn returning(&self) -> mir::Location {
        self.returning
    }
    pub(crate) fn into_chains(self) -> Vec<ChainEvidence<'tcx>> {
        self.body.into_chains()
    }
}

pub(crate) struct SelectedOwnedBody<'tcx> {
    shape: source::Shape<'tcx>,
    guard: mir::Location,
    parameter: mir::Local,
    paths: [SelectionPath<'tcx>; 2],
}
impl<'tcx> SelectedOwnedBody<'tcx> {
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        source::read(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        Self::from_body(tcx, owner, &body)
    }
    fn from_body(tcx: TyCtxt<'tcx>, owner: LocalDefId, body: &mir::Body<'tcx>) -> Result<Self> {
        let shape = source::read(tcx, owner)?;
        let traces = flow::selection(tcx, body)?;
        let parameter = body
            .args_iter()
            .nth(shape.parameter_index)
            .ok_or(Error::Argument)?;
        let guard = traces[0]
            .branch
            .as_ref()
            .ok_or(Error::ControlFlow)?
            .location;
        let paths = [
            path(tcx, owner, &shape, body, &traces[0], parameter)?,
            path(tcx, owner, &shape, body, &traces[1], parameter)?,
        ];
        let bindings = |path: &SelectionPath<'tcx>| -> HashMap<_, _> {
            path.chains()
                .iter()
                .flat_map(|chain| chain.bindings().iter().copied())
                .collect()
        };
        // All source binding identities (including the common destination)
        // have the same MIR local on both paths, even though its origin differs.
        if bindings(&paths[0]) != bindings(&paths[1]) {
            return Err(Error::SourceIdentity);
        }
        Ok(Self {
            shape,
            guard,
            parameter,
            paths,
        })
    }
    pub(crate) fn branch(&self) -> &'tcx hir::Expr<'tcx> {
        self.shape.branch
    }
    pub(crate) fn condition(&self) -> &'tcx hir::Expr<'tcx> {
        self.shape.condition
    }
    pub(crate) fn operands(&self) -> [&'tcx hir::Expr<'tcx>; 2] {
        self.shape.operands
    }
    pub(crate) fn binding(&self) -> HirId {
        self.shape.binding
    }
    pub(crate) fn parameter(&self) -> (HirId, mir::Local) {
        (self.shape.parameter, self.parameter)
    }
    pub(crate) fn guard(&self) -> mir::Location {
        self.guard
    }
    pub(crate) fn paths(&self) -> &[SelectionPath<'tcx>; 2] {
        &self.paths
    }
    pub(crate) fn into_paths(self) -> [SelectionPath<'tcx>; 2] {
        self.paths
    }
}

fn path<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    shape: &source::Shape<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &flow::Trace<'_, 'tcx>,
    parameter: mir::Local,
) -> Result<SelectionPath<'tcx>> {
    let branch = trace.branch.as_ref().ok_or(Error::ControlFlow)?;
    let plan = super::source::read_selection(tcx, owner, branch.outcome)?;
    if plan.scopes.blocks().count() != 1 || plan.scopes.read_scope() != shape.root.hir_id {
        return Err(Error::Scope);
    }
    let mut used = flow::flags::assignments(trace);
    values::boolean_parameter(
        tcx,
        body,
        trace,
        branch.discriminator,
        branch.location,
        parameter,
        &mut used,
    )?;
    let matched = relations::validate_path(tcx, owner, &plan, body, trace, used)?;
    let chosen = plan
        .chains
        .iter()
        .position(|c| c.bindings.last() == Some(&shape.binding))
        .ok_or(Error::SourceIdentity)?;
    let transfer = *matched.chains[chosen]
        .moves
        .last()
        .ok_or(Error::MoveGraph)?;
    if !trace.before(branch.location, transfer)
        || trace
            .calls
            .iter()
            .any(|(location, _)| !trace.before(*location, branch.location))
        || matched
            .chains
            .iter()
            .flat_map(|c| &c.moves)
            .any(|&location| location != transfer && !trace.before(location, branch.location))
        || trace.cleanup.iter().any(|d| {
            !trace.before(matched.chains[chosen].drop, d.location())
                || !trace.before(d.definition(), d.location())
        })
    {
        return Err(Error::ControlFlow);
    }
    let returning = matched.returning;
    Ok(SelectionPath {
        outcome: branch.outcome,
        body: MultipleOwnedBody::from_matched(plan, matched)?,
        decisions: trace.cleanup.clone(),
        returning,
    })
}
