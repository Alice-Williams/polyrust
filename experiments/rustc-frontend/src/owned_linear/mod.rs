//! Closed source/producer correspondence, not target code or a borrow checker.
mod exits;
mod flow;
// The separate proof drivers select different public evidence entry points.
#[allow(dead_code)]
pub(crate) mod multiple;
#[cfg(owned_linear_proof)]
#[path = "../../test/owned_linear/mutations.rs"]
pub(crate) mod mutations;
mod relations;
mod scopes;
pub(crate) use scopes::ScopeEvidence;
#[cfg(owned_scope_proof)]
#[path = "../../test/owned_scopes/mutations.rs"]
pub(crate) mod scope_mutations;
mod source;
mod values;

use crate::owned_source::{BoxConstructionInput, ConstructionError};
use rustc_hir::{HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LinearError {
    Signature,
    BodyShape,
    SourceIdentity,
    Scope,
    Constructor(ConstructionError),
    Record(crate::owned_source::record::RecordError),
    BoxedRecord(crate::owned_source::boxed_record::BoxedRecordError),
    Owner,
    Phase,
    ControlFlow,
    Call,
    Argument,
    Ambiguous,
    Assignment,
    MoveGraph,
    Read,
    Drop,
    Budget,
}
type Result<T> = std::result::Result<T, LinearError>;

/// Fields cannot be assembled by a target consumer. Compiler-session lifetime
/// comes from canonical constructor HIR/types, never persisted local numbers.
pub(crate) struct LinearOwnedBody<'tcx> {
    constructor: BoxConstructionInput<'tcx>,
    parameter: (HirId, mir::Local),
    bindings: Vec<(HirId, mir::Local)>,
    moves: Vec<mir::Location>,
    scope: HirId,
    scopes: ScopeEvidence<'tcx>,
    scalar_read: mir::Location,
    drop: mir::Location,
}

impl<'tcx> LinearOwnedBody<'tcx> {
    /// Caller must be the successful-analysis boundary. No arbitrary MIR input.
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        let plan = source::read(tcx, owner)?;
        Self::from_plan(tcx, owner, plan)
    }
    pub(crate) fn read_tail_scopes(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        let plan = source::read_tail_scopes(tcx, owner)?;
        Self::from_plan(tcx, owner, plan)
    }
    fn from_plan(tcx: TyCtxt<'tcx>, owner: LocalDefId, plan: source::Plan<'tcx>) -> Result<Self> {
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let matched = relations::validate(tcx, &plan, &body)?;
        Ok(Self {
            parameter: (plan.parameter, matched.parameter),
            bindings: plan.bindings.into_iter().zip(matched.owners).collect(),
            moves: matched.moves,
            scope: plan.scope,
            scopes: plan.scopes,
            scalar_read: matched.scalar_read,
            drop: matched.drop,
            constructor: plan.constructor,
        })
    }
    pub(crate) fn parameter(&self) -> (HirId, mir::Local) {
        self.parameter
    }
    pub(crate) fn bindings(&self) -> &[(HirId, mir::Local)] {
        &self.bindings
    }
    pub(crate) fn scope(&self) -> HirId {
        self.scope
    }
    pub(crate) fn scopes(&self) -> &ScopeEvidence<'tcx> {
        &self.scopes
    }
    /// Entry n moves bindings[n] into bindings[n + 1], in source chain order.
    pub(crate) fn moves(&self) -> &[mir::Location] {
        &self.moves
    }
    pub(crate) fn scalar_read(&self) -> mir::Location {
        self.scalar_read
    }
    pub(crate) fn drop_location(&self) -> mir::Location {
        self.drop
    }
    pub(crate) fn into_construction(self) -> BoxConstructionInput<'tcx> {
        self.constructor
    }
}
