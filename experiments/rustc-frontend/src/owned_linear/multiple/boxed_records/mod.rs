//! Private evidence for one Box of scalar fields; not a target admission token.
mod aggregate;
#[cfg(boxed_record_flow_proof)]
#[path = "../../../../test/boxed_record_flow/mutations.rs"]
pub(crate) mod mutations;
mod relations;
mod source;
pub(crate) use super::super::exits::Exit as SourceExit;
use super::super::{Result, scopes::ScopeFacts};
use crate::owned_source::{boxed_record::ScalarRecordBoxInput, scalar_record::ScalarField};
use rustc_abi::FieldIdx;
use rustc_hir::{self as hir, HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PayloadTransfer {
    Move,
    Copy,
}
pub(crate) struct FieldProducer<'tcx> {
    index: FieldIdx,
    initializer: &'tcx hir::Expr<'tcx>,
    parameter: (HirId, mir::Local),
    staging: (mir::Local, mir::Location),
}
impl<'tcx> FieldProducer<'tcx> {
    pub(crate) fn index(&self) -> FieldIdx {
        self.index
    }
    pub(crate) fn initializer(&self) -> &'tcx hir::Expr<'tcx> {
        self.initializer
    }
    pub(crate) fn parameter(&self) -> (HirId, mir::Local) {
        self.parameter
    }
    pub(crate) fn staging(&self) -> (mir::Local, mir::Location) {
        self.staging
    }
}
pub(crate) struct BoxedRecordBody<'tcx> {
    constructor: ScalarRecordBoxInput<'tcx>,
    literal: &'tcx hir::Expr<'tcx>,
    record: (HirId, mir::Local),
    fields: Vec<FieldProducer<'tcx>>,
    argument: mir::Local,
    transfer: PayloadTransfer,
    aggregate: mir::Location,
    movement: mir::Location,
    owners: Vec<(HirId, mir::Local)>,
    moves: Vec<mir::Location>,
    selected: FieldIdx,
    pointer: mir::Local,
    read: mir::Location,
    drop: mir::Location,
    returning: mir::Location,
    scopes: ScopeFacts<'tcx>,
}
impl<'tcx> BoxedRecordBody<'tcx> {
    /// The only production constructor queries canonical HIR and actual MIR.
    pub(crate) fn read(tcx: TyCtxt<'tcx>, owner: LocalDefId) -> Result<Self> {
        let plan = source::read(tcx, owner)?;
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let matched = relations::validate(tcx, &plan, &body)?;
        let parameters: Vec<_> = body.args_iter().collect();
        let fields = plan
            .fields
            .iter()
            .zip(matched.payload.fields)
            .map(|(f, staging)| {
                let position = plan
                    .parameters
                    .iter()
                    .position(|p| p.0 == f.parameter)
                    .unwrap();
                FieldProducer {
                    index: f.index,
                    initializer: f.expression,
                    parameter: (f.parameter, parameters[position]),
                    staging,
                }
            })
            .collect();
        Ok(Self {
            constructor: plan.constructor,
            literal: plan.literal,
            record: (plan.record, matched.payload.record),
            fields,
            argument: matched.payload.argument,
            transfer: matched.payload.transfer,
            aggregate: matched.payload.aggregate,
            movement: matched.payload.movement,
            owners: plan.owners.into_iter().zip(matched.owners).collect(),
            moves: matched.moves,
            selected: plan.selected,
            pointer: matched.pointer,
            read: matched.read,
            drop: matched.drop,
            returning: matched.returning,
            scopes: plan.scopes,
        })
    }
    pub(crate) fn literal(&self) -> &'tcx hir::Expr<'tcx> {
        self.literal
    }
    pub(crate) fn record(&self) -> (HirId, mir::Local) {
        self.record
    }
    pub(crate) fn fields(&self) -> &[FieldProducer<'tcx>] {
        &self.fields
    }
    pub(crate) fn argument(&self) -> mir::Local {
        self.argument
    }
    pub(crate) fn transfer(&self) -> PayloadTransfer {
        self.transfer
    }
    pub(crate) fn aggregate(&self) -> mir::Location {
        self.aggregate
    }
    pub(crate) fn movement(&self) -> mir::Location {
        self.movement
    }
    pub(crate) fn owners(&self) -> &[(HirId, mir::Local)] {
        &self.owners
    }
    pub(crate) fn moves(&self) -> &[mir::Location] {
        &self.moves
    }
    pub(crate) fn selected(&self) -> &ScalarField<'tcx> {
        &self.constructor.payload().fields()[self.selected.as_usize()]
    }
    pub(crate) fn pointer(&self) -> mir::Local {
        self.pointer
    }
    pub(crate) fn scalar_read(&self) -> mir::Location {
        self.read
    }
    pub(crate) fn drop_location(&self) -> mir::Location {
        self.drop
    }
    pub(crate) fn scopes(&self) -> &ScopeFacts<'tcx> {
        &self.scopes
    }
    pub(crate) fn exit(&self) -> SourceExit<'tcx> {
        self.scopes.exit()
    }
    pub(crate) fn returning(&self) -> mir::Location {
        self.returning
    }
    pub(crate) fn into_construction(self) -> ScalarRecordBoxInput<'tcx> {
        self.constructor
    }
}
