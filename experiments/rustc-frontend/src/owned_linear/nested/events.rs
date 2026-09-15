//! Typed correspondence projections, constructed only inside the nested reader.
use super::path::SourcePlace;
use rustc_abi::FieldIdx;
use rustc_hir::HirId;
use rustc_middle::mir;

pub(crate) struct Construction {
    pub(super) parameter: (HirId, mir::Local),
    pub(super) binding: (HirId, mir::Local),
    pub(super) call: mir::Location,
    pub(super) staging: Vec<mir::Location>,
}
impl Construction {
    pub(crate) fn parameter(&self) -> (HirId, mir::Local) {
        self.parameter
    }
    pub(crate) fn binding(&self) -> (HirId, mir::Local) {
        self.binding
    }
    pub(crate) fn call(&self) -> mir::Location {
        self.call
    }
    pub(crate) fn scalar_staging(&self) -> &[mir::Location] {
        &self.staging
    }
}
pub(crate) struct Field<'tcx> {
    pub(super) initializer: HirId,
    pub(super) index: FieldIdx,
    pub(super) source: (HirId, mir::Place<'tcx>),
    pub(super) staging: mir::Place<'tcx>,
    pub(super) movement: mir::Location,
    pub(super) destination: mir::Place<'tcx>,
}
impl<'tcx> Field<'tcx> {
    pub(crate) fn initializer(&self) -> HirId {
        self.initializer
    }
    pub(crate) fn index(&self) -> FieldIdx {
        self.index
    }
    pub(crate) fn source(&self) -> (HirId, mir::Place<'tcx>) {
        self.source
    }
    pub(crate) fn staging(&self) -> mir::Place<'tcx> {
        self.staging
    }
    pub(crate) fn movement(&self) -> mir::Location {
        self.movement
    }
    pub(crate) fn destination(&self) -> mir::Place<'tcx> {
        self.destination
    }
}
pub(crate) struct Aggregate<'tcx> {
    pub(super) binding: HirId,
    pub(super) destination: mir::Place<'tcx>,
    pub(super) location: mir::Location,
    pub(super) fields: Vec<Field<'tcx>>,
}
impl<'tcx> Aggregate<'tcx> {
    pub(crate) fn binding(&self) -> HirId {
        self.binding
    }
    pub(crate) fn destination(&self) -> mir::Place<'tcx> {
        self.destination
    }
    pub(crate) fn location(&self) -> mir::Location {
        self.location
    }
    /// Canonical initializer evaluation order, never operand declaration order.
    pub(crate) fn fields(&self) -> &[Field<'tcx>] {
        &self.fields
    }
}
pub(crate) struct Movement<'tcx> {
    pub(super) source: SourcePlace<'tcx>,
    pub(super) actual: mir::Place<'tcx>,
    pub(super) destination: (HirId, mir::Local),
    pub(super) location: mir::Location,
}
impl<'tcx> Movement<'tcx> {
    pub(crate) fn source(&self) -> &SourcePlace<'tcx> {
        &self.source
    }
    pub(crate) fn actual_source(&self) -> mir::Place<'tcx> {
        self.actual
    }
    pub(crate) fn destination(&self) -> (HirId, mir::Local) {
        self.destination
    }
    pub(crate) fn location(&self) -> mir::Location {
        self.location
    }
}
pub(crate) struct Leaf<'tcx> {
    pub(super) constructor: HirId,
    pub(super) source: SourcePlace<'tcx>,
    pub(super) actual: mir::Place<'tcx>,
    pub(super) cleanup: mir::Location,
}
impl<'tcx> Leaf<'tcx> {
    pub(crate) fn constructor(&self) -> HirId {
        self.constructor
    }
    pub(crate) fn source(&self) -> &SourcePlace<'tcx> {
        &self.source
    }
    pub(crate) fn actual(&self) -> mir::Place<'tcx> {
        self.actual
    }
    /// May be a shared whole-Inner Drop, never a fabricated per-field event.
    pub(crate) fn cleanup_location(&self) -> mir::Location {
        self.cleanup
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CleanupKind {
    Leaf,
    InnerRecord,
}
pub(crate) struct Cleanup<'tcx> {
    pub(super) kind: CleanupKind,
    pub(super) source: SourcePlace<'tcx>,
    pub(super) actual: mir::Place<'tcx>,
    pub(super) location: mir::Location,
    pub(super) constructors: Vec<HirId>,
}
impl<'tcx> Cleanup<'tcx> {
    pub(crate) fn kind(&self) -> CleanupKind {
        self.kind
    }
    pub(crate) fn source(&self) -> &SourcePlace<'tcx> {
        &self.source
    }
    pub(crate) fn actual(&self) -> mir::Place<'tcx> {
        self.actual
    }
    pub(crate) fn location(&self) -> mir::Location {
        self.location
    }
    /// Recursive declaration order inside this actual MIR cleanup event.
    pub(crate) fn constructors(&self) -> &[HirId] {
        &self.constructors
    }
}
