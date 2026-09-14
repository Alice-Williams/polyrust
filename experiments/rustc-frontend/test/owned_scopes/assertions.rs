//! Independent canonical-block and compiler-place assertions for every fixture.
use crate::{
    owned_linear::LinearOwnedBody,
    owned_source::{BoxConstructionInput, Builder, OwnedBoxConstruction},
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::{self as hir, def::DefKind, def_id::DefId};
use rustc_middle::{mir, ty::TyCtxt};
use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Copy)]
struct Observe;
impl Mapping for Observe {
    type Capability = OwnedBoxConstruction;
    type Context<'tcx> = TyCtxt<'tcx>;
    type Output = DefId;
    fn lower<'tcx>(
        &self,
        tcx: &mut TyCtxt<'tcx>,
        input: BoxConstructionInput<'tcx>,
    ) -> Result<DefId, String> {
        assert_eq!(input.call().hir_id.owner.def_id, input.owner());
        assert_eq!(
            tcx.typeck(input.owner()).expr_ty(input.argument()),
            tcx.types.i32
        );
        assert_eq!(
            tcx.fn_sig(input.constructor())
                .instantiate(*tcx, input.arguments())
                .skip_binder()
                .output(),
            input.result()
        );
        Ok(input.constructor())
    }
}

pub(super) fn check(tcx: TyCtxt<'_>) {
    let mut seen = BTreeSet::new();
    for owner in tcx
        .hir_body_owners()
        .filter(|owner| tcx.def_kind(*owner) == DefKind::Fn)
    {
        let name = tcx.def_path_str(owner);
        let expected: Option<(usize, &[usize], usize)> = match name.as_str() {
            "root" => Some((1, &[0], 0)),
            "inward" => Some((2, &[0, 1], 1)),
            "read_wrapper" => Some((2, &[0], 0)),
            "inner_construction" => Some((2, &[1], 1)),
            "many" => Some((4, &[0, 1, 2], 2)),
            "shadow" => Some((3, &[0, 1], 1)),
            "empty_wrappers" => Some((4, &[2], 2)),
            "bad_sibling" | "bad_escape" | "bad_branch" | "bad_early" | "bad_multiple" => None,
            _ => panic!("unexpected fixture {name}"),
        };
        let result = LinearOwnedBody::read_tail_scopes(tcx, owner);
        if let Some((depth, binding_depths, drop_depth)) = expected {
            let proof = result.unwrap_or_else(|error| {
                let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
                panic!("{name}: {error:?}; {:?}", body.basic_blocks)
            });
            assert_eq!(LinearOwnedBody::read(tcx, owner).is_ok(), name == "root");
            let scopes: Vec<_> = proof.scopes().blocks().collect();
            assert_eq!(scopes.len(), depth);
            assert_eq!(
                scopes.iter().map(|s| s.0).collect::<HashSet<_>>().len(),
                depth
            );
            assert_eq!(scopes[0], (proof.scope(), None));
            for pair in scopes.windows(2) {
                assert_eq!(pair[1].1, Some(pair[0].0));
            }
            let hir::ExprKind::Block(mut block, None) = tcx.hir_body_owned_by(owner).value.kind
            else {
                unreachable!()
            };
            for (index, &(scope, _)) in scopes.iter().enumerate() {
                assert_eq!(scope, block.hir_id);
                if index + 1 < depth {
                    let hir::ExprKind::Block(child, None) = block.expr.unwrap().kind else {
                        panic!("tail block")
                    };
                    block = child;
                }
            }
            assert_eq!(proof.scopes().read_scope(), scopes[depth - 1].0);
            assert_eq!(proof.scopes().drop_scope(), scopes[drop_depth].0);
            assert_eq!(proof.bindings().len(), binding_depths.len());
            for ((&(id, scope), &(binding, _)), &index) in proof
                .scopes()
                .bindings()
                .iter()
                .zip(proof.bindings())
                .zip(binding_depths)
            {
                assert_eq!(id, binding);
                assert_eq!(scope, scopes[index].0);
                let hir::Node::Block(declaration_block) = tcx.hir_node(scope) else {
                    panic!("canonical block")
                };
                assert!(declaration_block.stmts.iter().any(|statement| matches!(statement.kind, hir::StmtKind::Let(local) if matches!(local.pat.kind, hir::PatKind::Binding(_, actual, _, _) if actual == id))));
            }
            let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
            assert_eq!(proof.parameter().1, body.args_iter().next().unwrap());
            assert_eq!(proof.moves().len() + 1, proof.bindings().len());
            for (pair, location) in proof.bindings().windows(2).zip(proof.moves()) {
                let mir::StatementKind::Assign(assignment) =
                    &body.basic_blocks[location.block].statements[location.statement_index].kind
                else {
                    panic!("move")
                };
                assert_eq!(assignment.0, mir::Place::from(pair[1].1));
                assert!(
                    matches!(&assignment.1, mir::Rvalue::Use(mir::Operand::Move(source), _) if *source == mir::Place::from(pair[0].1))
                );
            }
            let location = proof.drop_location();
            assert!(
                matches!(body.basic_blocks[location.block].terminator().kind, mir::TerminatorKind::Drop {place, ..} if place == mir::Place::from(proof.bindings().last().unwrap().1))
            );
            let read = proof.scalar_read();
            assert!(
                matches!(&body.basic_blocks[read.block].statements[read.statement_index].kind, mir::StatementKind::Assign(pair) if pair.0.local == mir::RETURN_PLACE)
            );
            #[cfg(owned_scope_proof)]
            if name == "many" {
                crate::owned_linear::scope_mutations::check(tcx, owner);
            }
            let mut context = tcx;
            assert_eq!(
                Builder::new()
                    .construction(Observe)
                    .build()
                    .mapping()
                    .lower(&mut context, proof.into_construction())
                    .unwrap(),
                tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap()
            );
        } else {
            assert!(result.is_err(), "unsupported {name}");
        }
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 12);
}
