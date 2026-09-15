//! Independent canonical identities and selected-path runtime obligations.
use crate::owned_linear::multiple::{
    MultipleOwnedBody,
    early::EarlyOwnedBody,
    guarded::GuardedOwnedBody,
    returns::ReturningOwnedBody,
    selection::{Outcome, SelectedOwnedBody, SourceExit},
};
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
};
use rustc_middle::ty::TyCtxt;
use std::collections::BTreeSet;

#[path = "paths.rs"]
mod paths;

pub(super) fn check(tcx: TyCtxt<'_>) {
    let mut seen = BTreeSet::new();
    for owner in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let name = tcx.def_path_str(owner);
        let expected = match name.as_str() {
            "select" | "explicit" | "shadow" => Some((0, [2, 1], 2)),
            "moved" => Some((0, [1, 2], 3)),
            "reordered" => Some((2, [0, 1], 2)),
            "middle" => Some((1, [0, 2], 2)),
            "extra_before" | "extra_after" => Some((0, [2, 1], 3)),
            "same" | "branch_statement" | "allocation" | "after_move" | "negated"
            | "nested_exit" | "wrong_read" | "assignment" | "two_bools" | "constant" | "tail" => {
                None
            }
            _ => panic!("unknown selection fixture {name}"),
        };
        let result = SelectedOwnedBody::read(tcx, owner);
        if let Some((guard_index, selected, chain_count)) = expected {
            let proof = result.unwrap_or_else(|e| panic!("{name}: {e:?}"));
            assert!(MultipleOwnedBody::read(tcx, owner).is_err());
            assert!(ReturningOwnedBody::read(tcx, owner).is_err());
            assert!(GuardedOwnedBody::read(tcx, owner).is_err());
            assert!(EarlyOwnedBody::read(tcx, owner).is_err());
            let source = tcx.hir_body_owned_by(owner);
            let hir::ExprKind::Block(root, None) = source.value.kind else {
                panic!("root")
            };
            let (statements, exit) = match root.expr {
                Some(value) => (root.stmts, value),
                None => {
                    let (last, prefix) = root.stmts.split_last().unwrap();
                    let hir::StmtKind::Semi(value) = last.kind else {
                        panic!("return")
                    };
                    (prefix, value)
                }
            };
            let hir::StmtKind::Let(declaration) = statements.last().unwrap().kind else {
                panic!("selection binding")
            };
            let hir::PatKind::Binding(_, binding, _, _) = declaration.pat.kind else {
                panic!("binding")
            };
            assert_eq!(proof.binding(), binding);
            assert!(std::ptr::eq(proof.branch(), declaration.init.unwrap()));
            let hir::ExprKind::If(condition, yes, Some(no)) = proof.branch().kind else {
                panic!("branch")
            };
            assert!(std::ptr::eq(proof.condition(), condition));
            for (actual, arm) in proof.operands().into_iter().zip([no, yes]) {
                let hir::ExprKind::Block(block, None) = arm.kind else {
                    panic!("arm")
                };
                assert!(block.stmts.is_empty());
                assert!(std::ptr::eq(actual, block.expr.unwrap()));
            }
            let hir::PatKind::Binding(_, guard, _, _) = source.params[guard_index].pat.kind else {
                panic!("guard")
            };
            let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
            assert_eq!(
                proof.parameter(),
                (guard, body.args_iter().nth(guard_index).unwrap())
            );
            for (index, path) in proof.paths().iter().enumerate() {
                assert_eq!(
                    path.outcome(),
                    if index == 0 {
                        Outcome::False
                    } else {
                        Outcome::True
                    }
                );
                assert_eq!(path.chains().len(), chain_count);
                assert_eq!(
                    path.scopes().blocks().collect::<Vec<_>>(),
                    vec![(root.hir_id, None)]
                );
                assert_eq!(path.scopes().read_scope(), root.hir_id);
                crate::exit_consumer::check(tcx, owner, root.hir_id, path.exit(), path.returning());
                let value = match (path.exit(), exit.kind) {
                    (
                        SourceExit::Return { expression, value },
                        hir::ExprKind::Ret(Some(expected)),
                    ) => {
                        assert!(std::ptr::eq(expression, exit));
                        assert!(std::ptr::eq(value, expected));
                        value
                    }
                    (SourceExit::Tail(value), _) => {
                        assert!(std::ptr::eq(value, exit));
                        value
                    }
                    _ => panic!("exit kind"),
                };
                assert_eq!(
                    matches!(path.exit(), SourceExit::Return { .. }),
                    matches!(name.as_str(), "explicit" | "moved")
                );
                let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = value.kind else {
                    panic!("read")
                };
                let hir::ExprKind::Path(ref qpath) = operand.kind else {
                    panic!("read operand")
                };
                assert_eq!(
                    tcx.typeck(owner).qpath_res(qpath, operand.hir_id),
                    Res::Local(binding)
                );
                assert_eq!(path.scalar_read().0, binding);
                let chosen = path
                    .chains()
                    .iter()
                    .find(|c| c.bindings().last().unwrap().0 == binding)
                    .unwrap();
                let hir::PatKind::Binding(_, parameter, _, _) =
                    source.params[selected[index]].pat.kind
                else {
                    panic!("selected parameter")
                };
                assert_eq!(chosen.parameter().0, parameter);
                assert_eq!(path.drop_order()[0], binding);
                assert!(path.chains().iter().all(|c| c.drop_scope() == root.hir_id));
                paths::check(tcx, &body, &proof, index);
            }
            #[cfg(owned_selection_proof)]
            if name == "moved" {
                crate::owned_linear::multiple::selection::mutations::check(tcx, owner);
            }
            for path in proof.into_paths() {
                crate::compatibility::chains(tcx, path.into_chains());
            }
        } else {
            assert!(result.is_err(), "admitted {name}");
            if name == "tail" {
                crate::compatibility::tail(tcx, owner);
            }
        }
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 19);
}
