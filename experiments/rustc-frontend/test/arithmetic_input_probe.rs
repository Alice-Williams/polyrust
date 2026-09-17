//! Observe actual rustc evidence, with copied-node and cross-function controls.
use super::*;
impl<'tcx> ArithmeticInput<'tcx> {
    pub(crate) fn probe(
        &self,
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
    ) -> &'tcx hir::Expr<'tcx> {
        for expression in [self.source, self.left, self.right] {
            assert!(
                matches!(tcx.hir_node(expression.hir_id), hir::Node::Expr(node) if std::ptr::eq(node, expression))
            );
            assert!(checked.expr_adjustments(expression).is_empty());
            assert!(matches!(
                checked.expr_ty(expression).kind(),
                ty::Float(ty::FloatTy::F64)
            ));
        }
        assert!(checked.type_dependent_def_id(self.source.hir_id).is_none());
        let hir::ExprKind::Binary(operator, left, right) = self.source.kind else {
            panic!("binary source")
        };
        assert!(std::ptr::eq(left, self.left) && std::ptr::eq(right, self.right));
        let expected = match operator.node {
            hir::BinOpKind::Add => FloatingArithmeticOperator::Add,
            hir::BinOpKind::Sub => FloatingArithmeticOperator::Subtract,
            hir::BinOpKind::Mul => FloatingArithmeticOperator::Multiply,
            hir::BinOpKind::Div => FloatingArithmeticOperator::Divide,
            _ => panic!("arithmetic source"),
        };
        assert_eq!(self.operator, expected);
        let copied = hir::Expr {
            hir_id: self.source.hir_id,
            kind: self.source.kind,
            span: self.source.span,
        };
        assert!(Self::read(tcx, checked, &copied).is_err());
        let other = tcx
            .hir_body_owners()
            .find(|id| {
                *id != self.source.hir_id.owner.def_id && tcx.def_kind(*id) == hir::def::DefKind::Fn
            })
            .unwrap();
        assert!(Self::read(tcx, tcx.typeck(other), self.source).is_err());
        assert!(self.require_context(tcx, tcx.typeck(other)).is_err());
        self.require_context(tcx, checked).unwrap();
        eprintln!("ARITHMETIC_INPUT\t{:?}", self.operator);
        self.source
    }
}
