//! Read-only compiler witness, copied-node and wrong-context controls.
use super::*;
use rustc_hir::def::DefKind;
impl<'tcx> FloatingInput<'tcx> {
    pub(crate) fn probe(&self, tcx: TyCtxt<'tcx>, checked: &TypeckResults<'tcx>) {
        for expression in [self.source, self.operand] {
            assert!(
                matches!(tcx.hir_node(expression.hir_id), hir::Node::Expr(node)
                if std::ptr::eq(node, expression))
            );
            assert!(checked.expr_adjustments(expression).is_empty());
            assert!(matches!(
                checked.expr_ty(expression).kind(),
                ty::Float(ty::FloatTy::F64)
            ));
        }
        let hir::ExprKind::Unary(hir::UnOp::Neg, operand) = self.source.kind else {
            panic!("unary")
        };
        assert!(std::ptr::eq(operand, self.operand));
        assert!(checked.type_dependent_def_id(self.source.hir_id).is_none());
        let copied = hir::Expr {
            hir_id: self.source.hir_id,
            kind: self.source.kind,
            span: self.source.span,
        };
        assert!(Self::read(tcx, checked, &copied).is_err());
        let other = tcx
            .hir_body_owners()
            .find(|id| *id != self.source.hir_id.owner.def_id && tcx.def_kind(*id) == DefKind::Fn)
            .unwrap();
        assert!(Self::read(tcx, tcx.typeck(other), self.source).is_err());
        assert!(self.require_context(tcx, tcx.typeck(other)).is_err());
        self.require_context(tcx, checked).unwrap();
        eprintln!("FLOATING_INPUT\tF64");
    }
}
