//! Independent canonical HIR and actual MIR exit assertions.
use crate::owned_linear::{
    LinearOwnedBody,
    multiple::{MultipleOwnedBody, returns::ReturningOwnedBody},
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
            "root_value" | "root_semi" | "nested_value" | "nested_semi" | "moved" | "outer" => {
                Some(0)
            }
            "inner" | "shadow" | "interleaved" => Some(1),
            "tail" | "conditional" | "suffix" | "scalar" | "return_block" => None,
            _ => panic!("unexpected fixture {name}"),
        };
        let result = ReturningOwnedBody::read(tcx, owner);
        if let Some(index) = expected {
            let proof = result.unwrap_or_else(|error| {
                panic!(
                    "{name}: {error:?}; {:?}",
                    tcx.mir_drops_elaborated_and_const_checked(owner)
                        .borrow()
                        .basic_blocks
                )
            });
            assert!(MultipleOwnedBody::read(tcx, owner).is_err());
            assert!(LinearOwnedBody::read_tail_scopes(tcx, owner).is_err());
            let hir_body = tcx.hir_body_owned_by(owner);
            let hir::ExprKind::Block(mut block, None) = hir_body.value.kind else {
                panic!("root")
            };
            while let Some(expr) = block.expr {
                if let hir::ExprKind::Block(child, None) = expr.kind {
                    block = child;
                } else {
                    break;
                }
            }
            assert_eq!(
                block.expr.is_some(),
                matches!(name.as_str(), "root_value" | "nested_value" | "inner")
            );
            let expression = match block.expr {
                Some(expression) => expression,
                None => match block.stmts.last().unwrap().kind {
                    hir::StmtKind::Semi(expression) => expression,
                    _ => panic!("final return statement"),
                },
            };
            let hir::ExprKind::Ret(Some(value)) = expression.kind else {
                panic!("canonical return")
            };
            assert!(std::ptr::eq(expression, proof.exit().expression()));
            assert!(std::ptr::eq(value, proof.exit().value()));
            assert_eq!(block.hir_id, proof.exit().scope());
            let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = value.kind else {
                panic!("read")
            };
            let hir::ExprKind::Path(path) = &operand.kind else {
                panic!("owner path")
            };
            assert_eq!(
                tcx.typeck(owner).qpath_res(path, operand.hir_id),
                Res::Local(proof.body().scalar_read().0)
            );
            let chain = proof
                .body()
                .chains()
                .iter()
                .find(|c| c.bindings().last().unwrap().0 == proof.body().scalar_read().0)
                .unwrap();
            let hir::PatKind::Binding(_, parameter, _, _) = hir_body.params[index].pat.kind else {
                panic!("parameter")
            };
            assert_eq!(chain.parameter().0, parameter);
            let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
            let returning = proof.exit().location();
            assert_eq!(
                returning.statement_index,
                body.basic_blocks[returning.block].statements.len()
            );
            assert!(matches!(
                body.basic_blocks[returning.block].terminator().kind,
                TerminatorKind::Return
            ));
            let mut current = mir::START_BLOCK;
            let mut drops = Vec::new();
            let mut visited = BTreeSet::new();
            loop {
                assert!(visited.insert(current));
                let data = &body.basic_blocks[current];
                match data.terminator().kind {
                    TerminatorKind::Goto { target }
                    | TerminatorKind::Call {
                        target: Some(target),
                        ..
                    } => current = target,
                    TerminatorKind::Drop { place, target, .. } => {
                        drops.push(place.local);
                        current = target;
                    }
                    TerminatorKind::Return => {
                        assert_eq!(current, returning.block);
                        break;
                    }
                    _ => panic!("unexpected path"),
                }
            }
            let expected_drops: Vec<_> = proof
                .body()
                .drop_order()
                .iter()
                .map(|id| {
                    proof
                        .body()
                        .chains()
                        .iter()
                        .find(|c| c.bindings().last().unwrap().0 == *id)
                        .unwrap()
                        .bindings()
                        .last()
                        .unwrap()
                        .1
                })
                .collect();
            assert_eq!(drops, expected_drops);
            if name == "outer" || name == "interleaved" {
                assert_ne!(chain.drop_scope(), proof.exit().scope());
            }
            #[cfg(owned_return_proof)]
            if name == "interleaved" {
                crate::owned_linear::multiple::returns::mutations::check(tcx, owner);
            }
            let (body, exit) = proof.into_parts();
            assert_eq!(body.scopes().read_scope(), exit.scope());
            crate::compatibility::mapping(tcx, body);
        } else {
            assert!(result.is_err(), "admitted {name}");
            if name == "tail" {
                crate::compatibility::tail(tcx, owner);
            }
        }
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 14);
}
