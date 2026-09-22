//! One canonical, unadjusted i32-to-i64 cast, not an arbitrary source spelling.
use super::Capability;
use rustc_hir as hir;
use rustc_middle::ty::{self, TyCtxt, TypeckResults};
#[cfg(widening_ast_probe)]
#[path = "../../test/widening_input_probe.rs"]
mod input_probe;

pub(crate) struct SignedWidening;
pub(crate) struct WideningInput<'tcx> {
    source: &'tcx hir::Expr<'tcx>,
    operand: &'tcx hir::Expr<'tcx>,
}
impl Capability for SignedWidening {
    type Input<'tcx> = WideningInput<'tcx>;
}
impl<'tcx> WideningInput<'tcx> {
    pub(crate) fn discover(
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &hir::Expr<'tcx>,
    ) -> Result<Option<Self>, String> {
        if !matches!(expression.kind, hir::ExprKind::Cast(..)) {
            return Ok(None);
        }
        let hir::Node::Expr(canonical) = tcx.hir_node(expression.hir_id) else {
            return Err("signed widening requires a canonical HIR expression".into());
        };
        if !std::ptr::eq(canonical, expression)
            || !std::ptr::eq(checked, tcx.typeck(expression.hir_id.owner.def_id))
        {
            return Err(
                "signed widening requires its original checked function and HIR node".into(),
            );
        }
        let hir::ExprKind::Cast(operand, _) = canonical.kind else {
            unreachable!()
        };
        if !matches!(checked.expr_ty(operand).kind(), ty::Int(ty::IntTy::I32))
            || !matches!(checked.expr_ty(canonical).kind(), ty::Int(ty::IntTy::I64))
            || checked.type_dependent_def_id(canonical.hir_id).is_some()
            || !checked.expr_adjustments(canonical).is_empty()
            || !checked.expr_adjustments(operand).is_empty()
        {
            return Err(
                "signed widening supports only an unadjusted i32 operand cast to i64".into(),
            );
        }
        Ok(Some(Self {
            source: canonical,
            operand,
        }))
    }
    pub(crate) fn require_context(
        &self,
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
    ) -> Result<(), String> {
        let current = Self::discover(tcx, checked, self.source)?
            .ok_or("signed widening requires its original cast")?;
        if !std::ptr::eq(current.operand, self.operand) {
            return Err("signed widening operand identity differs".into());
        }
        Ok(())
    }
    pub(crate) fn operand(&self) -> &'tcx hir::Expr<'tcx> {
        self.operand
    }
}
