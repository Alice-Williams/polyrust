//! Direct comparison fixture calls are resolved by compiler identity, not spelling.
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::DefId,
};
use rustc_middle::ty::{self, TypeckResults};
pub(super) fn operands<'tcx>(
    checked: &TypeckResults<'tcx>,
    source: &'tcx hir::Expr<'tcx>,
) -> Option<(hir::BinOpKind, [DefId; 2])> {
    let hir::ExprKind::Binary(operator, left, right) = source.kind else {
        panic!("binary")
    };
    if !matches!(checked.expr_ty(left).kind(), ty::Float(ty::FloatTy::F64)) {
        return None;
    }
    assert_eq!(checked.expr_ty(left), checked.expr_ty(right));
    let identity = |operand: &hir::Expr<'tcx>| {
        assert!(checked.expr_adjustments(operand).is_empty());
        let hir::ExprKind::Call(callee, _) = operand.kind else {
            panic!("fixture call")
        };
        let hir::ExprKind::Path(path) = &callee.kind else {
            panic!("direct callee")
        };
        let Res::Def(DefKind::Fn, identity) = checked.qpath_res(path, callee.hir_id) else {
            panic!("function")
        };
        identity
    };
    Some((operator.node, [identity(left), identity(right)]))
}
