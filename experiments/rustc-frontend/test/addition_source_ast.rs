//! Fixture-only structural discovery, independent of production capability readers.
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
};
use rustc_middle::ty::{TyCtxt, TypeckResults};

pub(super) fn operands<'tcx>(
    tcx: TyCtxt<'tcx>,
    checked: &TypeckResults<'tcx>,
    value: &'tcx hir::Expr<'tcx>,
) -> Option<(&'tcx hir::Expr<'tcx>, &'tcx hir::Expr<'tcx>)> {
    let (definition, left, right) = match value.kind {
        hir::ExprKind::MethodCall(_, left, [right], _) => (
            checked.type_dependent_def_id(value.hir_id).unwrap(),
            left,
            right,
        ),
        hir::ExprKind::Call(callee, [left, right]) => {
            let hir::ExprKind::Path(path) = &callee.kind else {
                return None;
            };
            let Res::Def(DefKind::AssocFn, definition) = checked.qpath_res(path, callee.hir_id)
            else {
                return None;
            };
            (definition, left, right)
        }
        _ => return None,
    };
    assert!(!definition.is_local());
    assert_eq!(tcx.item_name(definition).as_str(), "wrapping_add");
    Some((left, right))
}
