//! Generic source inputs must not depend on a target AST scope.
use super::ControlInput;
use portable_backend_c::ast::CScopeRef;
use rustc_hir as hir;

#[allow(dead_code)]
fn must_not_compile<'tcx>(expression: &'tcx hir::Expr<'tcx>, scope: CScopeRef) {
    let _ = ControlInput {
        expression,
        parent: Some(scope),
        completion: super::ControlCompletion::Return,
    };
}
