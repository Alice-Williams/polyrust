//! Read-only compiler evidence and negative canonical/context controls.
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::DefId,
    intravisit::{self, Visitor},
};
use rustc_middle::ty::{TyCtxt, TypeckResults};

pub(super) fn calls<'tcx>(
    checked: &TypeckResults<'tcx>,
    receiver: &'tcx hir::Expr<'tcx>,
) -> Vec<DefId> {
    struct Calls<'a, 'tcx> {
        checked: &'a TypeckResults<'tcx>,
        identities: Vec<DefId>,
    }
    impl<'tcx> Visitor<'tcx> for Calls<'_, 'tcx> {
        fn visit_expr(&mut self, expression: &'tcx hir::Expr<'tcx>) {
            intravisit::walk_expr(self, expression);
            if let hir::ExprKind::Call(callee, _) = expression.kind
                && let hir::ExprKind::Path(path) = &callee.kind
                && let Res::Def(DefKind::Fn, identity) = self.checked.qpath_res(path, callee.hir_id)
            {
                self.identities.push(identity);
            }
        }
    }
    let mut calls = Calls {
        checked,
        identities: Vec::new(),
    };
    calls.visit_expr(receiver);
    calls.identities
}

pub(super) fn checked_input<'tcx>(
    tcx: TyCtxt<'tcx>,
    checked: &TypeckResults<'tcx>,
    input: &crate::source_capabilities::FloatingInput<'tcx>,
) {
    input.probe(tcx, checked);
}
