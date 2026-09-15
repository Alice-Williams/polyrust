//! Ordinary consumer checks for canonical source exits and full MIR locations.
use crate::owned_linear::SourceExit;
use rustc_hir::{self as hir, HirId, def_id::LocalDefId};
use rustc_middle::{mir, ty::TyCtxt};

pub(crate) fn check<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    scope: HirId,
    exit: SourceExit<'tcx>,
    returning: mir::Location,
) {
    assert_eq!(scope.owner.def_id, owner);
    let hir::Node::Block(block) = tcx.hir_node(scope) else {
        panic!("exit scope is not a canonical block");
    };
    match exit {
        SourceExit::Tail(value) => {
            canonical(tcx, owner, value);
            assert!(std::ptr::eq(block.expr.expect("tail value"), value));
            assert!(!matches!(value.kind, hir::ExprKind::Ret(_)));
        }
        SourceExit::Return { expression, value } => {
            canonical(tcx, owner, expression);
            canonical(tcx, owner, value);
            let expected = block.expr.unwrap_or_else(|| {
                let hir::StmtKind::Semi(last) = block.stmts.last().expect("return statement").kind
                else {
                    panic!("explicit return is not the block ending");
                };
                last
            });
            assert!(std::ptr::eq(expected, expression));
            assert!(
                matches!(expression.kind, hir::ExprKind::Ret(Some(actual)) if std::ptr::eq(actual, value))
            );
        }
    }
    let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    assert_eq!(body.source.def_id(), owner.to_def_id());
    assert_eq!(
        body.phase,
        mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup)
    );
    let data = &body.basic_blocks[returning.block];
    assert_eq!(returning.statement_index, data.statements.len());
    assert!(matches!(
        data.terminator().kind,
        mir::TerminatorKind::Return
    ));
}

fn canonical<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId, expression: &'tcx hir::Expr<'tcx>) {
    assert_eq!(expression.hir_id.owner.def_id, owner);
    let hir::Node::Expr(actual) = tcx.hir_node(expression.hir_id) else {
        panic!("exit expression is not compiler-owned");
    };
    assert!(std::ptr::eq(actual, expression));
}
