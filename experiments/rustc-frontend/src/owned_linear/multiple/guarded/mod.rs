//! One authenticated Boolean guard and two complete normal ownership paths.
pub(super) mod source;
pub(crate) use super::super::exits::Outcome;
use super::super::{LinearError as Error, Result, flow, scopes::ScopeFacts, values};
use super::{ChainEvidence, MultipleOwnedBody, relations};
use rustc_hir::{self as hir, HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};
use std::collections::HashSet;

pub(crate) struct GuardEvidence<'tcx> {
    branch: &'tcx hir::Expr<'tcx>,
    condition: &'tcx hir::Expr<'tcx>,
    parameter: (HirId, mir::Local),
    location: mir::Location,
}
impl<'tcx> GuardEvidence<'tcx> {
    pub(crate) fn branch(&self) -> &'tcx hir::Expr<'tcx> {
        self.branch
    }
    pub(crate) fn condition(&self) -> &'tcx hir::Expr<'tcx> {
        self.condition
    }
    pub(crate) fn parameter(&self) -> (HirId, mir::Local) {
        self.parameter
    }
    pub(crate) fn location(&self) -> mir::Location {
        self.location
    }
}
pub(crate) struct GuardedPath<'tcx> {
    outcome: Outcome,
    chains: Vec<ChainEvidence<'tcx>>,
    scopes: ScopeFacts<'tcx>,
    read: (HirId, mir::Location),
    drops: Vec<HirId>,
    returning: mir::Location,
}
impl<'tcx> GuardedPath<'tcx> {
    pub(crate) fn outcome(&self) -> Outcome {
        self.outcome
    }
    pub(crate) fn chains(&self) -> &[ChainEvidence<'tcx>] {
        &self.chains
    }
    pub(crate) fn scopes(&self) -> &ScopeFacts<'tcx> {
        &self.scopes
    }
    pub(crate) fn scalar_read(&self) -> (HirId, mir::Location) {
        self.read
    }
    pub(crate) fn drop_order(&self) -> &[HirId] {
        &self.drops
    }
    pub(crate) fn returning(&self) -> mir::Location {
        self.returning
    }
    pub(crate) fn into_chains(self) -> Vec<ChainEvidence<'tcx>> {
        self.chains
    }
}
pub(crate) struct GuardedOwnedBody<'tcx> {
    guard: GuardEvidence<'tcx>,
    paths: [GuardedPath<'tcx>; 2],
}
impl<'tcx> GuardedOwnedBody<'tcx> {
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        source::read(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        Self::from_body(tcx, owner, &body)
    }
    fn from_body(tcx: TyCtxt<'tcx>, owner: LocalDefId, body: &mir::Body<'tcx>) -> Result<Self> {
        let shape = source::read(tcx, owner)?;
        let [no, yes] = flow::split(body)?;
        let location = no.branch.as_ref().ok_or(Error::ControlFlow)?.location;
        let no = path(tcx, owner, &shape, body, &no)?;
        let yes = path(tcx, owner, &shape, body, &yes)?;
        let parameter = body
            .args_iter()
            .nth(shape.parameter_index)
            .ok_or(Error::Argument)?;
        Ok(Self {
            guard: GuardEvidence {
                branch: shape.branch,
                condition: shape.condition,
                parameter: (shape.parameter, parameter),
                location,
            },
            paths: [no, yes],
        })
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

#[cfg(owned_guard_proof)]
#[path = "../../../../test/owned_guarded/mutations.rs"]
pub(crate) mod mutations;
fn path<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    shape: &source::Shape<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &flow::Trace<'_, 'tcx>,
) -> Result<GuardedPath<'tcx>> {
    let branch = trace.branch.as_ref().ok_or(Error::ControlFlow)?;
    let plan = super::source::read_guarded(tcx, owner, branch.outcome)?;
    if plan.scopes.read_scope() != shape.arm(branch.outcome)
        || plan.scopes.blocks().next().map(|b| b.0) != Some(shape.root.hir_id)
    {
        return Err(Error::Scope);
    }
    let parameter = body
        .args_iter()
        .nth(shape.parameter_index)
        .ok_or(Error::Argument)?;
    let mut used = HashSet::new();
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
    if !trace.before(branch.location, matched.read)
        || trace
            .calls
            .iter()
            .any(|&(location, _)| !trace.before(location, branch.location))
        || matched
            .chains
            .iter()
            .flat_map(|c| &c.moves)
            .any(|&location| !trace.before(location, branch.location))
    {
        return Err(Error::ControlFlow);
    }
    let returning = matched.returning;
    let MultipleOwnedBody {
        chains,
        scopes,
        read,
        drops,
    } = MultipleOwnedBody::from_matched(plan, matched)?;
    Ok(GuardedPath {
        outcome: branch.outcome,
        chains,
        scopes,
        read,
        drops,
        returning,
    })
}
