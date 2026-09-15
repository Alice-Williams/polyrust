//! Authenticate one concrete standard constructor using compiler identities.
use crate::source_capabilities::Capability;
use rustc_hir::{
    self as hir,
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{GenericArgsRef, Ty, TyCtxt};

pub(crate) struct OwnedBoxConstruction;
impl Capability for OwnedBoxConstruction {
    type Input<'tcx> = BoxConstructionInput<'tcx>;
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConstructionError {
    WrongOwner,
    NonCanonicalNode,
    NotDirectCall,
    NotStandardConstructor,
    UnsupportedPayload,
    SignatureMismatch,
}

/// Operation identity only: callers must already be inside successful analysis.
/// This is not a standalone source-analysis or whole-body ownership certificate.
pub(crate) struct BoxConstructionInput<'tcx> {
    owner: LocalDefId,
    call: &'tcx hir::Expr<'tcx>,
    argument: &'tcx hir::Expr<'tcx>,
    constructor: DefId,
    arguments: GenericArgsRef<'tcx>,
    result: Ty<'tcx>,
}

impl<'tcx> BoxConstructionInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        owner: LocalDefId,
        call: &hir::Expr<'tcx>,
    ) -> Result<Self, ConstructionError> {
        let checked = super::standard_box::Call::read(tcx, owner, call)?;
        if checked.payload != tcx.types.i32 {
            return Err(ConstructionError::UnsupportedPayload);
        }
        Ok(Self {
            owner: checked.owner,
            call: checked.call,
            argument: checked.argument,
            constructor: checked.constructor,
            arguments: checked.arguments,
            result: checked.result,
        })
    }

    pub(crate) fn owner(&self) -> LocalDefId {
        self.owner
    }
    pub(crate) fn call(&self) -> &'tcx hir::Expr<'tcx> {
        self.call
    }
    pub(crate) fn argument(&self) -> &'tcx hir::Expr<'tcx> {
        self.argument
    }
    pub(crate) fn constructor(&self) -> DefId {
        self.constructor
    }
    pub(crate) fn arguments(&self) -> GenericArgsRef<'tcx> {
        self.arguments
    }
    pub(crate) fn result(&self) -> Ty<'tcx> {
        self.result
    }
}
