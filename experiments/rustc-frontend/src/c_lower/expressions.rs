//! Traversal selects executable mappings; their owners define operation rules.
use super::{
    Reader, Result, c,
    capabilities::{
        BorrowInput, CallInput, ComparisonInput, DirectCalls, LiteralInput, LiteralValues, Mapping,
        PlaceInput, ResolvedPlaces, ScalarComparisons, SharedBorrows, Supports,
    },
};
use portable_backend_c::ast::{CPlace, CValue};
use rustc_hir as hir;

impl<'tcx> Reader<'tcx> {
    pub(super) fn expr(&mut self, value: &'tcx hir::Expr<'tcx>) -> Result<CValue> {
        self.ty(self.checked.expr_ty(value))?;
        if !self.checked.expr_adjustments(value).is_empty() {
            let place = self.place(value)?;
            return c(self.expressions().read(place));
        }
        match value.kind {
            hir::ExprKind::Call(..) => {
                let mapping = Supports::<DirectCalls>::mapping(&self.mappings);
                mapping.lower(self, CallInput(value))
            }
            hir::ExprKind::Path(_)
            | hir::ExprKind::Field(..)
            | hir::ExprKind::Unary(hir::UnOp::Deref, _) => {
                let place = self.place(value)?;
                c(self.expressions().read(place))
            }
            hir::ExprKind::Lit(_) | hir::ExprKind::Unary(hir::UnOp::Neg, _) => {
                let mapping = Supports::<LiteralValues>::mapping(&self.mappings);
                mapping.lower(self, LiteralInput(value))
            }
            hir::ExprKind::AddrOf(hir::BorrowKind::Ref, hir::Mutability::Not, _) => {
                let mapping = Supports::<SharedBorrows>::mapping(&self.mappings);
                mapping.lower(self, BorrowInput(value))
            }
            hir::ExprKind::Binary(..) => {
                let mapping = Supports::<ScalarComparisons>::mapping(&self.mappings);
                mapping.lower(self, ComparisonInput(value))
            }
            _ => Err("C expression mapping is not implemented".into()),
        }
    }

    pub(super) fn place(&mut self, value: &'tcx hir::Expr<'tcx>) -> Result<CPlace> {
        let mapping = Supports::<ResolvedPlaces>::mapping(&self.mappings);
        mapping.lower(self, PlaceInput(value))
    }
}
