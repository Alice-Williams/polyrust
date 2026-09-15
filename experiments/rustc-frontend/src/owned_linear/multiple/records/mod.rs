//! Query-only record ownership certificates; no target heap admission.
mod aggregate;
mod constructors;
#[cfg(owned_fields_proof)]
#[path = "../../../../test/owned_fields/mutations.rs"]
pub(crate) mod mutations;
mod relations;
mod source;
use super::super::{Result, scopes::ScopeFacts};
use crate::owned_source::{BoxConstructionInput, record::RecordConstructionInput};
use rustc_hir::{HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};
pub(crate) use source::SourcePlace;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Transfer<'tcx> {
    Move {
        location: mir::Location,
    },
    IntoField {
        staging: mir::Place<'tcx>,
        movement: mir::Location,
        aggregate: mir::Location,
    },
}
pub(crate) struct ChainEvidence<'tcx> {
    constructor: BoxConstructionInput<'tcx>,
    parameter: (HirId, mir::Local),
    places: Vec<(SourcePlace, mir::Place<'tcx>)>,
    transfers: Vec<Transfer<'tcx>>,
    drop: mir::Location,
    drop_scope: HirId,
}
impl<'tcx> ChainEvidence<'tcx> {
    pub(crate) fn parameter(&self) -> (HirId, mir::Local) {
        self.parameter
    }
    pub(crate) fn places(&self) -> &[(SourcePlace, mir::Place<'tcx>)] {
        &self.places
    }
    pub(crate) fn transfers(&self) -> &[Transfer<'tcx>] {
        &self.transfers
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
pub(crate) struct RecordOwnedBody<'tcx> {
    record: RecordConstructionInput<'tcx>,
    aggregate: (HirId, mir::Place<'tcx>, mir::Location),
    chains: Vec<ChainEvidence<'tcx>>,
    scopes: ScopeFacts<'tcx>,
    read: (HirId, mir::Location),
    drops: Vec<SourcePlace>,
}
impl<'tcx> RecordOwnedBody<'tcx> {
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        let plan = source::read(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let matched = relations::validate(tcx, owner, &plan, &body)?;
        let drop_scope = plan.scopes.read_scope();
        let chains = plan
            .chains
            .into_iter()
            .zip(matched.chains)
            .map(|(source, actual)| ChainEvidence {
                constructor: source.constructor,
                parameter: (source.parameter, actual.parameter),
                places: source.places.into_iter().zip(actual.places).collect(),
                transfers: actual.transfers,
                drop: actual.drop,
                drop_scope,
            })
            .collect();
        Ok(Self {
            record: plan.record,
            aggregate: (plan.binding, matched.aggregate.0, matched.aggregate.1),
            chains,
            scopes: plan.scopes,
            read: (plan.read, matched.read),
            drops: matched.drops,
        })
    }
    pub(crate) fn aggregate(&self) -> (HirId, mir::Place<'tcx>, mir::Location) {
        self.aggregate
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
    pub(crate) fn drop_order(&self) -> &[SourcePlace] {
        &self.drops
    }
    pub(crate) fn into_operations(
        self,
    ) -> (RecordConstructionInput<'tcx>, Vec<ChainEvidence<'tcx>>) {
        (self.record, self.chains)
    }
}
