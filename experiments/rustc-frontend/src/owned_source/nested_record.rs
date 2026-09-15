//! Canonical nested construction identity, never a whole-body certificate.
#[path = "nested_layout.rs"]
mod layout;
use crate::source_capabilities::Capability;
#[cfg(nested_record_proof)]
pub(crate) use layout::oracles as layout_oracles;
pub(crate) use layout::{Field, FieldKind, RecordLayout};
use rustc_abi::FieldIdx;
use rustc_hir::{self as hir, def_id::LocalDefId};
use rustc_middle::ty::{Ty, TyCtxt};
use std::collections::HashSet;

pub(crate) struct OwnedNestedRecordConstruction;
impl Capability for OwnedNestedRecordConstruction {
    type Input<'tcx> = NestedRecordConstructionInput<'tcx>;
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NestedError {
    WrongOwner,
    NonCanonicalNode,
    NotCompleteStruct,
    UnsupportedRecord,
    UnsupportedField,
    DepthBudget,
    FieldBudget,
    RecursiveRecord,
    FlatRecord,
    Adjustment,
    FieldCoverage,
    InitializerType,
}
pub(crate) struct Initializer<'tcx> {
    index: FieldIdx,
    expression: &'tcx hir::Expr<'tcx>,
}
impl<'tcx> Initializer<'tcx> {
    pub(crate) fn index(&self) -> FieldIdx {
        self.index
    }
    pub(crate) fn expression(&self) -> &'tcx hir::Expr<'tcx> {
        self.expression
    }
}
pub(crate) struct NestedRecordConstructionInput<'tcx> {
    owner: LocalDefId,
    expression: &'tcx hir::Expr<'tcx>,
    layout: RecordLayout<'tcx>,
    initializers: Vec<Initializer<'tcx>>,
}
impl<'tcx> NestedRecordConstructionInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        owner: LocalDefId,
        expression: &hir::Expr<'tcx>,
    ) -> Result<Self, NestedError> {
        if expression.hir_id.owner.def_id != owner {
            return Err(NestedError::WrongOwner);
        }
        let hir::Node::Expr(canonical) = tcx.hir_node(expression.hir_id) else {
            return Err(NestedError::NonCanonicalNode);
        };
        if !std::ptr::eq(canonical, expression) {
            return Err(NestedError::NonCanonicalNode);
        }
        let hir::ExprKind::Struct(_, fields, hir::StructTailExpr::None) = canonical.kind else {
            return Err(NestedError::NotCompleteStruct);
        };
        let checked = tcx.typeck(owner);
        if !checked.expr_adjustments(canonical).is_empty() {
            return Err(NestedError::Adjustment);
        }
        let layout = layout::read(tcx, checked.expr_ty(canonical))?;
        if !layout
            .fields()
            .iter()
            .any(|field| matches!(field.kind(), FieldKind::Record(_)))
        {
            return Err(NestedError::FlatRecord);
        }
        if fields.len() != layout.fields().len() {
            return Err(NestedError::FieldCoverage);
        }
        let mut seen = HashSet::new();
        let mut initializers = Vec::new();
        for field in fields {
            let index = checked.field_index(field.hir_id);
            let declared = layout
                .fields()
                .get(index.as_usize())
                .ok_or(NestedError::FieldCoverage)?;
            if !seen.insert(index) {
                return Err(NestedError::FieldCoverage);
            }
            if !checked.expr_adjustments(field.expr).is_empty() {
                return Err(NestedError::Adjustment);
            }
            if checked.expr_ty(field.expr) != declared.ty() {
                return Err(NestedError::InitializerType);
            }
            initializers.push(Initializer {
                index,
                expression: field.expr,
            });
        }
        Ok(Self {
            owner,
            expression: canonical,
            layout,
            initializers,
        })
    }
    pub(crate) fn owner(&self) -> LocalDefId {
        self.owner
    }
    pub(crate) fn expression(&self) -> &'tcx hir::Expr<'tcx> {
        self.expression
    }
    pub(crate) fn layout(&self) -> &RecordLayout<'tcx> {
        &self.layout
    }
    pub(crate) fn field(&self, index: FieldIdx) -> Option<&Field<'tcx>> {
        self.layout.fields().get(index.as_usize())
    }
    pub(crate) fn result(&self) -> Ty<'tcx> {
        self.layout.ty()
    }
    /// Canonical source evaluation order, distinct from layout declaration order.
    pub(crate) fn initializers(&self) -> &[Initializer<'tcx>] {
        &self.initializers
    }
}
