//! Check real source exit kinds, scopes and actual per-outcome MIR cleanup.
use crate::owned_linear::{
    LinearOwnedBody,
    multiple::{
        MultipleOwnedBody,
        early::EarlyOwnedBody,
        guarded::{GuardedOwnedBody, Outcome, SourceExit},
        returns::ReturningOwnedBody,
    },
};
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
};
use rustc_middle::{
    mir::{self, TerminatorKind},
    ty::TyCtxt,
};
use std::collections::BTreeSet;

pub(super) fn check(tcx: TyCtxt<'_>) {
    let mut seen = BTreeSet::new();
    for owner in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let name = tcx.def_path_str(owner);
        let expected = match name.as_str() {
            "tail_value" | "explicit_value" | "explicit_semi" | "arm_value" | "moved" => {
                Some((0, [2, 1]))
            }
            "swapped" => Some((0, [1, 2])),
            "middle" => Some((1, [2, 0])),
            "unused" => Some((3, [0, 2])),
            "intervening" | "branch_move" | "nested_condition" | "with_else" | "value_call"
            | "suffix" | "continuation_block" | "negated" | "tail" => None,
            _ => panic!("unexpected fixture {name}"),
        };
        let result = EarlyOwnedBody::read(tcx, owner);
        if let Some((guard_index, selected)) = expected {
            let proof = result.unwrap_or_else(|e| {
                panic!(
                    "{name}: {e:?}; {:?}",
                    tcx.mir_drops_elaborated_and_const_checked(owner)
                        .borrow()
                        .basic_blocks
                )
            });
            assert!(GuardedOwnedBody::read(tcx, owner).is_err());
            assert!(MultipleOwnedBody::read(tcx, owner).is_err());
            assert!(ReturningOwnedBody::read(tcx, owner).is_err());
            assert!(LinearOwnedBody::read_tail_scopes(tcx, owner).is_err());
            let source = tcx.hir_body_owned_by(owner);
            let hir::ExprKind::Block(root, None) = source.value.kind else {
                panic!("root")
            };
            let (before, continuation) = match root.expr {
                Some(expr) => (root.stmts, expr),
                None => {
                    let (last, before) = root.stmts.split_last().unwrap();
                    let hir::StmtKind::Semi(expr) = last.kind else {
                        panic!("continuation")
                    };
                    (before, expr)
                }
            };
            let (hir::StmtKind::Expr(branch) | hir::StmtKind::Semi(branch)) =
                before.last().unwrap().kind
            else {
                panic!("if statement")
            };
            let hir::ExprKind::If(condition, arm, None) = branch.kind else {
                panic!("early if")
            };
            let hir::ExprKind::Block(arm, None) = arm.kind else {
                panic!("arm")
            };
            assert!(std::ptr::eq(proof.guard().branch(), branch));
            assert!(std::ptr::eq(proof.guard().condition(), condition));
            let hir::PatKind::Binding(_, parameter, _, _) = source.params[guard_index].pat.kind
            else {
                panic!("guard")
            };
            let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
            assert_eq!(
                proof.guard().parameter(),
                (parameter, body.args_iter().nth(guard_index).unwrap())
            );
            assert_eq!(arm.expr.is_some(), name == "arm_value");
            let arm_return = arm
                .expr
                .unwrap_or_else(|| match arm.stmts.last().unwrap().kind {
                    hir::StmtKind::Semi(expr) => expr,
                    _ => panic!("arm return"),
                });
            for (index, path) in proof.paths().iter().enumerate() {
                assert_eq!(
                    path.outcome(),
                    if index == 0 {
                        Outcome::False
                    } else {
                        Outcome::True
                    }
                );
                let source_exit = if index == 0 { continuation } else { arm_return };
                let value = match (path.exit(), source_exit.kind) {
                    (
                        SourceExit::Return { expression, value },
                        hir::ExprKind::Ret(Some(expected)),
                    ) => {
                        assert!(std::ptr::eq(expression, source_exit));
                        assert!(std::ptr::eq(value, expected));
                        value
                    }
                    (SourceExit::Tail(value), _) if index == 0 => {
                        assert!(std::ptr::eq(value, source_exit));
                        value
                    }
                    _ => panic!("wrong retained exit kind"),
                };
                if index == 0 {
                    assert_eq!(
                        matches!(path.exit(), SourceExit::Return { .. }),
                        matches!(name.as_str(), "explicit_value" | "explicit_semi")
                    );
                }
                let scope = if index == 0 { root.hir_id } else { arm.hir_id };
                assert_eq!(path.scopes().read_scope(), scope);
                crate::exit_consumer::check(tcx, owner, scope, path.exit(), path.returning());
                assert_eq!(path.scopes().blocks().count(), index + 1);
                assert_eq!(path.scopes().blocks().next().unwrap(), (root.hir_id, None));
                let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = value.kind else {
                    panic!("deref")
                };
                let hir::ExprKind::Path(ref qpath) = operand.kind else {
                    panic!("owner path")
                };
                assert_eq!(
                    tcx.typeck(owner).qpath_res(qpath, operand.hir_id),
                    Res::Local(path.scalar_read().0)
                );
                let chain = path
                    .chains()
                    .iter()
                    .find(|c| c.bindings().last().unwrap().0 == path.scalar_read().0)
                    .unwrap();
                let hir::PatKind::Binding(_, scalar, _, _) =
                    source.params[selected[index]].pat.kind
                else {
                    panic!("scalar")
                };
                assert_eq!(chain.parameter().0, scalar);
                assert_eq!(chain.drop_scope(), root.hir_id);
                let mut current = mir::START_BLOCK;
                let mut visited = BTreeSet::new();
                let mut drops = Vec::new();
                let mut switches = 0;
                loop {
                    assert!(visited.insert(current));
                    match body.basic_blocks[current].terminator().kind {
                        TerminatorKind::Goto { target }
                        | TerminatorKind::Call {
                            target: Some(target),
                            ..
                        } => current = target,
                        TerminatorKind::SwitchInt { ref targets, .. } => {
                            switches += 1;
                            assert_eq!(current, proof.guard().location().block);
                            current = targets.target_for_value(index as u128);
                        }
                        TerminatorKind::Drop { place, target, .. } => {
                            drops.push(place.local);
                            current = target;
                        }
                        TerminatorKind::Return => {
                            assert_eq!(current, path.returning().block);
                            break;
                        }
                        _ => panic!("unexpected path"),
                    }
                }
                assert_eq!(switches, 1);
                assert_eq!(drops.len(), 2);
                assert_eq!(
                    drops,
                    path.drop_order()
                        .iter()
                        .map(|id| path
                            .chains()
                            .iter()
                            .find(|c| c.bindings().last().unwrap().0 == *id)
                            .unwrap()
                            .bindings()
                            .last()
                            .unwrap()
                            .1)
                        .collect::<Vec<_>>()
                );
            }
            let (_, paths) = proof.into_parts();
            for path in paths {
                crate::compatibility::chains(tcx, path.into_chains());
            }
            #[cfg(owned_early_proof)]
            if name == "moved" {
                crate::owned_linear::multiple::early::mutations::check(tcx, owner);
            }
        } else {
            assert!(result.is_err(), "admitted {name}");
            if name == "tail" {
                crate::compatibility::tail(tcx, owner);
            }
        }
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 17);
}
