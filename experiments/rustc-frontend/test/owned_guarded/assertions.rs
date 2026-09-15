//! Trace both actual successors and compare canonical return-owner identities.
use crate::owned_linear::{
    LinearOwnedBody,
    multiple::{
        MultipleOwnedBody,
        guarded::{GuardedOwnedBody, Outcome},
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
        // Boolean parameter index, then false/true selected scalar indices.
        let expected = match name.as_str() {
            "choose" | "moved" | "renamed" | "no_semicolons" => Some((0, [2, 1])),
            "reversed" => Some((0, [1, 2])),
            "middle" => Some((1, [2, 0])),
            "unused" => Some((3, [0, 2])),
            "negated" | "alias" | "branch_move" | "branch_constructor" | "implicit" | "early"
            | "two_flags" | "computed" | "tail" => None,
            _ => panic!("unexpected fixture {name}"),
        };
        let result = GuardedOwnedBody::read(tcx, owner);
        if let Some((guard_index, selections)) = expected {
            let proof = result.unwrap_or_else(|e| {
                panic!(
                    "{name}: {e:?}; {:?}",
                    tcx.mir_drops_elaborated_and_const_checked(owner)
                        .borrow()
                        .basic_blocks
                )
            });
            assert!(MultipleOwnedBody::read(tcx, owner).is_err());
            assert!(ReturningOwnedBody::read(tcx, owner).is_err());
            assert!(LinearOwnedBody::read_tail_scopes(tcx, owner).is_err());
            let source = tcx.hir_body_owned_by(owner);
            let hir::ExprKind::Block(root, None) = source.value.kind else {
                panic!("root")
            };
            let branch = root.expr.unwrap();
            let hir::ExprKind::If(condition, yes, Some(no)) = branch.kind else {
                panic!("branch")
            };
            assert!(std::ptr::eq(proof.guard().branch(), branch));
            assert!(std::ptr::eq(proof.guard().condition(), condition));
            let hir::PatKind::Binding(_, guard, _, _) = source.params[guard_index].pat.kind else {
                panic!("guard parameter")
            };
            let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
            assert_eq!(
                proof.guard().parameter(),
                (guard, body.args_iter().nth(guard_index).unwrap())
            );
            let location = proof.guard().location();
            assert_eq!(
                location.statement_index,
                body.basic_blocks[location.block].statements.len()
            );
            assert!(matches!(
                body.basic_blocks[location.block].terminator().kind,
                TerminatorKind::SwitchInt { .. }
            ));
            assert_eq!(
                proof.paths()[0].returning(),
                proof.paths()[1].returning(),
                "shared final Return"
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
                let arm = if index == 0 { no } else { yes };
                let hir::ExprKind::Block(block, None) = arm.kind else {
                    panic!("arm")
                };
                assert_eq!(block.expr.is_some(), name == "no_semicolons");
                let ret = block
                    .expr
                    .unwrap_or_else(|| match block.stmts.last().unwrap().kind {
                        hir::StmtKind::Semi(expr) => expr,
                        _ => panic!("return statement"),
                    });
                let hir::ExprKind::Ret(Some(value)) = ret.kind else {
                    panic!("explicit return")
                };
                let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = value.kind else {
                    panic!("return value")
                };
                let hir::ExprKind::Path(ref qpath) = operand.kind else {
                    panic!("owner")
                };
                assert_eq!(
                    tcx.typeck(owner).qpath_res(qpath, operand.hir_id),
                    Res::Local(path.scalar_read().0)
                );
                assert_eq!(path.scopes().read_scope(), block.hir_id);
                crate::exit_consumer::check(
                    tcx,
                    owner,
                    path.scopes().read_scope(),
                    path.exit(),
                    path.returning(),
                );
                assert_eq!(
                    path.scopes().blocks().collect::<Vec<_>>(),
                    vec![(root.hir_id, None), (block.hir_id, Some(root.hir_id))]
                );
                let selected = path
                    .chains()
                    .iter()
                    .find(|c| c.bindings().last().unwrap().0 == path.scalar_read().0)
                    .unwrap();
                let hir::PatKind::Binding(_, parameter, _, _) =
                    source.params[selections[index]].pat.kind
                else {
                    panic!("scalar parameter")
                };
                assert_eq!(selected.parameter().0, parameter);
                assert_eq!(selected.drop_scope(), root.hir_id);
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
                            assert_eq!(current, location.block);
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
                        _ => panic!("unaccounted flow"),
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
            #[cfg(owned_guard_proof)]
            if name == "moved" {
                crate::owned_linear::multiple::guarded::mutations::check(tcx, owner);
            }
            for path in paths {
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
    assert_eq!(seen.len(), 16);
}
