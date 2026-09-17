//! Canonical checked input, context and immutable finite-bit witness controls.
use super::*;
impl<'tcx> LiteralInput<'tcx> {
    pub(crate) fn probe(&self, tcx: TyCtxt<'tcx>, checked: &TypeckResults<'tcx>) {
        let LiteralValue::F64(value) = self.value() else {
            return;
        };
        let source = self._expression;
        assert!(matches!(tcx.hir_node(source.hir_id), hir::Node::Expr(node)
            if std::ptr::eq(node, source)));
        assert!(matches!(
            checked.expr_ty(source).kind(),
            ty::Float(ty::FloatTy::F64)
        ));
        assert!(checked.expr_adjustments(source).is_empty());
        let copied = hir::Expr {
            hir_id: source.hir_id,
            kind: source.kind,
            span: source.span,
        };
        assert!(Self::read(tcx, checked, &copied).is_err());
        let other = tcx
            .hir_body_owners()
            .find(|id| {
                *id != source.hir_id.owner.def_id
                    && tcx.def_kind(*id) == rustc_hir::def::DefKind::Fn
            })
            .unwrap();
        assert!(Self::read(tcx, tcx.typeck(other), source).is_err());
        assert!(self.require_context(tcx, tcx.typeck(other)).is_err());
        self.require_context(tcx, checked).unwrap();
        eprintln!("BINARY64_INPUT\t{:016x}", value.to_bits());
    }
}
