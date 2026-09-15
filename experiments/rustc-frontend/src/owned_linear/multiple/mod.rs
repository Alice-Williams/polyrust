//! Private multi-owner evidence; compiler analysis, not target heap admission.
pub(crate) mod early;
pub(crate) mod guarded;
#[cfg(owned_multiple_proof)]
#[path = "../../../test/owned_multiple/mutations.rs"]
pub(crate) mod mutations;
pub(crate) mod records;
mod relations;
#[cfg(owned_multiple_proof)]
#[path = "../../../test/owned_multiple/renaming.rs"]
pub(crate) mod renaming;
mod residual;
pub(crate) mod returns;
pub(crate) mod selection;
mod source;
use super::{LinearError, Result, scopes::ScopeFacts};
use crate::owned_source::BoxConstructionInput;
use rustc_hir::{HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};

pub(crate) struct ChainEvidence<'tcx> {
    constructor: BoxConstructionInput<'tcx>,
    parameter: (HirId, mir::Local),
    bindings: Vec<(HirId, mir::Local)>,
    moves: Vec<mir::Location>,
    drop: mir::Location,
    drop_scope: HirId,
}
impl<'tcx> ChainEvidence<'tcx> {
    pub(crate) fn parameter(&self) -> (HirId, mir::Local) {
        self.parameter
    }
    pub(crate) fn bindings(&self) -> &[(HirId, mir::Local)] {
        &self.bindings
    }
    pub(crate) fn moves(&self) -> &[mir::Location] {
        &self.moves
    }
    pub(crate) fn drop_location(&self) -> mir::Location {
        self.drop
    }
    pub(crate) fn drop_scope(&self) -> HirId {
        self.drop_scope
    }
    pub(crate) fn into_construction(self) -> BoxConstructionInput<'tcx> {
        self.constructor
    }
}

pub(crate) struct MultipleOwnedBody<'tcx> {
    chains: Vec<ChainEvidence<'tcx>>,
    scopes: ScopeFacts<'tcx>,
    read: (HirId, mir::Location),
    drops: Vec<HirId>,
}
impl<'tcx> MultipleOwnedBody<'tcx> {
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        let plan = source::read(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let matched = relations::validate(tcx, owner, &plan, &body)?;
        Self::from_matched(plan, matched)
    }
    fn from_matched(plan: source::Plan<'tcx>, matched: relations::Matched) -> Result<Self> {
        let mut chains = Vec::new();
        for (source, actual) in plan.chains.into_iter().zip(matched.chains) {
            let last = *source.bindings.last().ok_or(LinearError::MoveGraph)?;
            let drop_scope = plan
                .scopes
                .bindings()
                .iter()
                .find(|item| item.0 == last)
                .ok_or(LinearError::Scope)?
                .1;
            chains.push(ChainEvidence {
                constructor: source.constructor,
                parameter: (source.parameter, actual.parameter),
                bindings: source.bindings.into_iter().zip(actual.owners).collect(),
                moves: actual.moves,
                drop: actual.drop,
                drop_scope,
            });
        }
        Ok(Self {
            chains,
            scopes: plan.scopes,
            read: (plan.read, matched.read),
            drops: matched.drops,
        })
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
    pub(crate) fn into_chains(self) -> Vec<ChainEvidence<'tcx>> {
        self.chains
    }
}
