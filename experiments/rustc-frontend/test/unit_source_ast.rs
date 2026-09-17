//! Independent source counts for the fixed unit probe fixture.
use rustc_hir::{
    self as hir,
    intravisit::{self, Visitor},
};
use rustc_middle::ty::{self, TypeckResults};

pub(super) fn counts<'tcx>(
    checked: &TypeckResults<'tcx>,
    expression: &'tcx hir::Expr<'tcx>,
) -> (usize, usize) {
    struct Calls<'a, 'tcx> {
        checked: &'a TypeckResults<'tcx>,
        effects: usize,
        temporaries: usize,
    }
    impl<'tcx> Visitor<'tcx> for Calls<'_, 'tcx> {
        fn visit_expr(&mut self, expression: &'tcx hir::Expr<'tcx>) {
            if let hir::ExprKind::Call(_, arguments) = expression.kind {
                let unit = matches!(self.checked.expr_ty(expression).kind(), ty::Tuple(fields) if fields.is_empty());
                self.effects += usize::from(unit);
                self.temporaries += arguments.len() + usize::from(!unit);
            }
            intravisit::walk_expr(self, expression);
        }
    }
    let mut calls = Calls {
        checked,
        effects: 0,
        temporaries: 0,
    };
    calls.visit_expr(expression);
    (calls.effects, calls.temporaries)
}
