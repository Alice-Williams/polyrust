//! Fixture oracle inspects the private evidence through read-only typed access.
use crate::{
    owned_linear::{LinearError, LinearOwnedBody},
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
        let expected = match name.as_str() {
            "zero" | "renamed" => Some(1),
            "one" | "shadow" => Some(2),
            "many" => Some(4),
            "bad_extra" | "bad_literal" | "bad_early" | "bad_branch" | "bad_unrelated"
            | "bad_parameter" | "bad_payload" => None,
            _ => panic!("unexpected fixture {name}"),
        };
        let result = LinearOwnedBody::read(tcx, owner);
        #[cfg(owned_linear_proof)]
        if name == "many" {
            crate::owned_linear::mutations::check(tcx, owner);
        }
        if let Some(count) = expected {
            let scoped = LinearOwnedBody::read_tail_scopes(tcx, owner).unwrap();
            assert_eq!(
                scoped.scopes().blocks().collect::<Vec<_>>(),
                vec![(scoped.scope(), None)]
            );
            assert_eq!(scoped.scopes().read_scope(), scoped.scope());
            assert_eq!(scoped.scopes().drop_scope(), scoped.scope());
            assert_eq!(scoped.scopes().bindings().len(), count);
            let proof = result.unwrap_or_else(|error| {
                let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
                panic!(
                    "{name}: {error:?}\nlocals: {:?}\nblocks: {:?}",
                    body.local_decls, body.basic_blocks
                );
            });
            assert_eq!(proof.bindings().len(), count);
            assert_eq!(
                proof
                    .bindings()
                    .iter()
                    .map(|item| item.0)
                    .collect::<HashSet<_>>()
                    .len(),
                count
            );
            assert_eq!(
                proof
                    .bindings()
                    .iter()
                    .map(|item| item.1)
                    .collect::<HashSet<_>>()
                    .len(),
                count
            );
            let hir::ExprKind::Block(block, None) = tcx.hir_body_owned_by(owner).value.kind else {
                unreachable!()
            };
            assert_eq!(proof.scope(), block.hir_id);
            assert_eq!(proof.parameter().0.owner.def_id, owner);
            let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
            assert_eq!(proof.moves().len() + 1, count);
            for (pair, location) in proof.bindings().windows(2).zip(proof.moves()) {
                let mir::StatementKind::Assign(assignment) =
                    &body.basic_blocks[location.block].statements[location.statement_index].kind
                else {
                    panic!("move assignment")
                };
                assert_eq!(assignment.0, mir::Place::from(pair[1].1));
                assert!(
                    matches!(&assignment.1, mir::Rvalue::Use(mir::Operand::Move(source), _) if *source == mir::Place::from(pair[0].1))
                );
            }
            assert_eq!(proof.parameter().1, body.args_iter().next().unwrap());
            let final_place = proof.bindings().last().unwrap().1;
            let drop = proof.drop_location();
            assert_eq!(
                drop.statement_index,
                body.basic_blocks[drop.block].statements.len()
            );
            assert!(
                matches!(body.basic_blocks[drop.block].terminator().kind, mir::TerminatorKind::Drop { place, .. } if place.local == final_place)
            );
            let read = proof.scalar_read();
            assert!(
                matches!(&body.basic_blocks[read.block].statements[read.statement_index].kind, mir::StatementKind::Assign(pair) if pair.0.local == mir::RETURN_PLACE)
            );
            if name == "shadow" {
                for &(id, _) in proof.bindings() {
                    let hir::Node::Pat(pattern) = tcx.hir_node(id) else {
                        panic!("binding pattern")
                    };
                    assert!(
                        matches!(pattern.kind, hir::PatKind::Binding(_, _, ident, _) if ident.name.as_str() == "value")
                    );
                }
            }
            let bindings = Builder::new().construction(Observe).build();
            let mut context = tcx;
            assert_eq!(
                bindings
                    .mapping()
                    .lower(&mut context, proof.into_construction())
                    .unwrap(),
                tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap()
            );
        } else {
            let error = match result {
                Err(error) => error,
                Ok(_) => panic!("admitted {name}"),
            };
            match name.as_str() {
                "bad_extra" | "bad_literal" | "bad_early" | "bad_branch" => {
                    assert_eq!(error, LinearError::BodyShape)
                }
                "bad_unrelated" | "bad_payload" => {
                    assert!(matches!(error, LinearError::Constructor(_)))
                }
                "bad_parameter" => assert_eq!(error, LinearError::Signature),
                _ => unreachable!(),
            }
        }
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 12);
}
