//! Query-only nested source/MIR correspondence; no target heap admission.
mod aggregate;
mod cleanup;
mod constructors;
pub(crate) mod events;
mod frame;
#[cfg(owned_nested_proof)]
#[path = "../../../test/nested_correspondence/mutations.rs"]
pub(crate) mod mutations;
pub(crate) mod path;
mod relations;
mod source;

use super::{Result, SourceExit, scopes::ScopeFacts};
use crate::owned_source::{
    BoxConstructionInput, nested_record::NestedRecordConstructionInput,
    record::RecordConstructionInput,
};
use rustc_hir::{HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};

pub(crate) struct NestedOwnedBody<'tcx> {
    plan: source::Plan<'tcx>,
    matched: relations::Matched<'tcx>,
}
impl<'tcx> NestedOwnedBody<'tcx> {
    /// The successful-analysis caller supplies only the compiler and owner;
    /// arbitrary MIR and operation-only evidence cannot certify a whole body.
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        let plan = source::read(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let matched = relations::validate(tcx, owner, &plan, &body)?;
        Ok(Self { plan, matched })
    }
    pub(crate) fn owner(&self) -> LocalDefId {
        self.plan.frame.owner
    }
    pub(crate) fn constructions(&self) -> &[events::Construction] {
        &self.matched.constructions
    }
    pub(crate) fn aggregates(&self) -> &[events::Aggregate<'tcx>; 2] {
        &self.matched.aggregates
    }
    pub(crate) fn movements(&self) -> &[events::Movement<'tcx>] {
        &self.matched.movements
    }
    pub(crate) fn bindings(&self) -> &[(HirId, mir::Local)] {
        &self.matched.bindings
    }
    /// Remaining constructor-rooted leaves in exact normal cleanup order.
    pub(crate) fn leaves(&self) -> &[events::Leaf<'tcx>] {
        &self.matched.leaves
    }
    pub(crate) fn cleanup(&self) -> &[events::Cleanup<'tcx>] {
        &self.matched.cleanup
    }
    pub(crate) fn scopes(&self) -> &ScopeFacts<'tcx> {
        &self.plan.frame.scopes
    }
    pub(crate) fn exit(&self) -> SourceExit<'tcx> {
        self.plan.frame.scopes.exit()
    }
    pub(crate) fn scalar_read(&self) -> (HirId, mir::Location) {
        (self.plan.read, self.matched.read)
    }
    pub(crate) fn returning(&self) -> mir::Location {
        self.matched.returning
    }
    pub(crate) fn box_inputs(&self) -> impl Iterator<Item = &BoxConstructionInput<'tcx>> {
        self.plan.constructions.iter().map(|c| &c.input)
    }
    pub(crate) fn record_inputs(
        &self,
    ) -> (
        &RecordConstructionInput<'tcx>,
        &NestedRecordConstructionInput<'tcx>,
    ) {
        (&self.plan.inner.1, &self.plan.outer.1)
    }
    pub(crate) fn into_operations(
        self,
    ) -> (
        Vec<BoxConstructionInput<'tcx>>,
        RecordConstructionInput<'tcx>,
        NestedRecordConstructionInput<'tcx>,
    ) {
        (
            self.plan
                .constructions
                .into_iter()
                .map(|c| c.input)
                .collect(),
            self.plan.inner.1,
            self.plan.outer.1,
        )
    }
}
