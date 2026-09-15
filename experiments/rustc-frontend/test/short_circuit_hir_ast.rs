//! Compiler-side call occurrence inventory; independent of target traversal.
use rustc_hir::{
    self as hir,
    intravisit::{self, Visitor},
};

pub fn calls(expression: &hir::Expr<'_>) -> usize {
    struct Count(usize);
    impl<'v> Visitor<'v> for Count {
        fn visit_expr(&mut self, expression: &'v hir::Expr<'v>) {
            if matches!(expression.kind, hir::ExprKind::Call(..)) {
                self.0 += 1;
            }
            intravisit::walk_expr(self, expression);
        }
    }
    let mut count = Count(0);
    count.visit_expr(expression);
    count.0
}
