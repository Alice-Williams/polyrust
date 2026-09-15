//! A separate capability, never a widening of scalar Box construction.
use super::{
    ConstructionError,
    scalar_record::{PayloadError, ScalarRecord},
    standard_box,
};
use crate::source_capabilities::Capability;
use rustc_hir::{
    self as hir,
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{GenericArgsRef, Ty, TyCtxt};

pub(crate) struct OwnedScalarRecordBoxConstruction;
impl Capability for OwnedScalarRecordBoxConstruction {
    type Input<'tcx> = ScalarRecordBoxInput<'tcx>;
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BoxedRecordError {
    Constructor(ConstructionError),
    Payload(PayloadError),
    Adjustment,
}
pub(crate) struct ScalarRecordBoxInput<'tcx> {
    call: standard_box::Call<'tcx>,
    payload: ScalarRecord<'tcx>,
}
impl<'tcx> ScalarRecordBoxInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        owner: LocalDefId,
        call: &hir::Expr<'tcx>,
    ) -> Result<Self, BoxedRecordError> {
        let call =
            standard_box::Call::read(tcx, owner, call).map_err(BoxedRecordError::Constructor)?;
        if !tcx.typeck(owner).expr_adjustments(call.call).is_empty() {
            return Err(BoxedRecordError::Adjustment);
        }
        let payload = ScalarRecord::read(tcx, call.payload).map_err(BoxedRecordError::Payload)?;
        Ok(Self { call, payload })
    }
    pub(crate) fn owner(&self) -> LocalDefId {
        self.call.owner
    }
    pub(crate) fn call(&self) -> &'tcx hir::Expr<'tcx> {
        self.call.call
    }
    pub(crate) fn argument(&self) -> &'tcx hir::Expr<'tcx> {
        self.call.argument
    }
    pub(crate) fn constructor(&self) -> DefId {
        self.call.constructor
    }
    pub(crate) fn arguments(&self) -> GenericArgsRef<'tcx> {
        self.call.arguments
    }
    pub(crate) fn result(&self) -> Ty<'tcx> {
        self.call.result
    }
    pub(crate) fn payload(&self) -> &ScalarRecord<'tcx> {
        &self.payload
    }
}
