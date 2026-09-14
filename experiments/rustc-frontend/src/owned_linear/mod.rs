//! Closed source/producer correspondence, not target code or a borrow checker.
mod flow;
#[cfg(owned_linear_proof)]
#[path = "../../test/owned_linear/mutations.rs"]
pub(crate) mod mutations;
mod relations;
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
    Constructor(ConstructionError),
    Owner,
    Phase,
    ControlFlow,
    Call,
    Argument,
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
    scalar_read: mir::Location,
    drop: mir::Location,
}

impl<'tcx> LinearOwnedBody<'tcx> {
    /// Caller must be the successful-analysis boundary. No arbitrary MIR input.
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        let plan = source::read(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let matched = relations::validate(tcx, &plan, &body)?;
        Ok(Self {
            parameter: (plan.parameter, matched.parameter),
            bindings: plan.bindings.into_iter().zip(matched.owners).collect(),
            moves: matched.moves,
            scope: plan.scope,
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
