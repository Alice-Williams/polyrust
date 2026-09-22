//! Hostile identity controls against the actual private source witness.
use super::*;
use rustc_hir::def::DefKind;
impl<'tcx> WideningInput<'tcx> {
    pub(crate) fn probe(&self, tcx: TyCtxt<'tcx>, checked: &TypeckResults<'tcx>) {
        for (expression, width) in [
            (self.source, ty::IntTy::I64),
            (self.operand, ty::IntTy::I32),
        ] {
            assert!(
                matches!(tcx.hir_node(expression.hir_id), hir::Node::Expr(node)
                if std::ptr::eq(node, expression))
            );
            assert!(checked.expr_adjustments(expression).is_empty());
            assert!(
                matches!(checked.expr_ty(expression).kind(), ty::Int(actual) if *actual == width)
            );
        }
        let hir::ExprKind::Cast(operand, _) = self.source.kind else {
            panic!("cast")
        };
        assert!(std::ptr::eq(operand, self.operand));
        let copied = hir::Expr {
            hir_id: self.source.hir_id,
            kind: self.source.kind,
            span: self.source.span,
        };
        assert!(Self::discover(tcx, checked, &copied).is_err());
        let other = tcx
            .hir_body_owners()
            .find(|id| *id != self.source.hir_id.owner.def_id && tcx.def_kind(*id) == DefKind::Fn)
            .unwrap();
        assert!(Self::discover(tcx, tcx.typeck(other), self.source).is_err());
        assert!(self.require_context(tcx, tcx.typeck(other)).is_err());
        // Wrong-width operand substitution is not allowed to reuse a valid cast's authority.
        let forged = Self {
            source: self.source,
            operand: self.source,
        };
        assert!(forged.require_context(tcx, checked).is_err());
        self.require_context(tcx, checked).unwrap();
        eprintln!("WIDENING_INPUT\tI32_I64");
    }
}
