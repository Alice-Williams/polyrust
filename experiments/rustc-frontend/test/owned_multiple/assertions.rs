//! Inspect actual compiler argument/place/move/drop relations, never spelling.
use crate::{
    owned_linear::{LinearError, LinearOwnedBody, multiple::MultipleOwnedBody},
    owned_source::{BoxConstructionInput, Builder, OwnedBoxConstruction},
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::{self as hir, def::DefKind, def_id::DefId};
use rustc_middle::{mir, ty::TyCtxt};
use std::collections::{BTreeSet, HashMap, HashSet};
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
        // Constructor parameter order, cleanup parameter order, read parameter.
        let expected: Option<(&[usize], &[usize], usize)> = match name.as_str() {
            "one" => Some((&[0], &[0], 0)),
            "two_first" | "outer_drop" => Some((&[0, 1], &[1, 0], 0)),
            "two_second" | "shadow" | "same_scope_shadow" => Some((&[0, 1], &[1, 0], 1)),
            "interleaved" | "nested" => Some((&[0, 1], &[0, 1], 1)),
            "unused_parameter" => Some((&[2, 0], &[0, 2], 0)),
            "repeated_anchor" | "bad_literal" | "bad_call" | "bad_scalar" | "bad_branch" => None,
            _ => panic!("unexpected fixture {name}"),
        };
        let result = MultipleOwnedBody::read(tcx, owner);
        #[cfg(owned_multiple_proof)]
        if name == "interleaved" {
            crate::owned_linear::multiple::mutations::check(tcx, owner);
        }
        #[cfg(owned_multiple_proof)]
        if name == "two_first" {
            crate::owned_linear::multiple::renaming::check(tcx, owner);
        }
        if let Some((constructors, drops, read_parameter)) = expected {
            let proof = result.unwrap_or_else(|error| {
                panic!(
                    "{name}: {error:?}; {:?}",
                    tcx.mir_drops_elaborated_and_const_checked(owner)
                        .borrow()
                        .basic_blocks
                )
            });
            let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
            let parameters: Vec<_> = body.args_iter().collect();
            let hir_parameters: Vec<_> = tcx
                .hir_body_owned_by(owner)
                .params
                .iter()
                .map(|p| {
                    let hir::PatKind::Binding(_, id, _, _) = p.pat.kind else {
                        unreachable!()
                    };
                    id
                })
                .collect();
            assert_eq!(proof.chains().len(), constructors.len());
            let mut ends = HashMap::new();
            let mut locals = HashSet::new();
            for (chain, &index) in proof.chains().iter().zip(constructors) {
                assert_eq!(
                    chain.parameter(),
                    (hir_parameters[index], parameters[index])
                );
                assert_eq!(chain.moves().len() + 1, chain.bindings().len());
                for &(_, local) in chain.bindings() {
                    assert!(locals.insert(local));
                }
                for (pair, location) in chain.bindings().windows(2).zip(chain.moves()) {
                    let mir::StatementKind::Assign(assignment) = &body.basic_blocks[location.block]
                        .statements[location.statement_index]
                        .kind
                    else {
                        panic!("move")
                    };
                    assert_eq!(assignment.0, mir::Place::from(pair[1].1));
                    assert!(
                        matches!(&assignment.1, mir::Rvalue::Use(mir::Operand::Move(p), _) if *p == mir::Place::from(pair[0].1))
                    );
                }
                let &(id, local) = chain.bindings().last().unwrap();
                assert!(ends.insert(id, (index, local)).is_none());
                assert_eq!(
                    proof
                        .scopes()
                        .bindings()
                        .iter()
                        .find(|item| item.0 == id)
                        .unwrap()
                        .1,
                    chain.drop_scope()
                );
                let location = chain.drop_location();
                assert!(
                    matches!(body.basic_blocks[location.block].terminator().kind, mir::TerminatorKind::Drop {place, ..} if place == mir::Place::from(local))
                );
            }
            assert_eq!(
                proof
                    .drop_order()
                    .iter()
                    .map(|id| ends[id].0)
                    .collect::<Vec<_>>(),
                drops
            );
            assert_eq!(ends[&proof.scalar_read().0].0, read_parameter);
            crate::exit_consumer::check(
                tcx,
                owner,
                proof.scopes().read_scope(),
                proof.exit(),
                proof.returning(),
            );
            let read = proof.scalar_read().1;
            assert!(
                matches!(&body.basic_blocks[read.block].statements[read.statement_index].kind, mir::StatementKind::Assign(pair) if pair.0.local == mir::RETURN_PLACE)
            );
            assert_eq!(
                proof.scopes().blocks().last().unwrap().0,
                proof.scopes().read_scope()
            );
            if name == "one" {
                let old = LinearOwnedBody::read(tcx, owner).unwrap();
                let nested = LinearOwnedBody::read_tail_scopes(tcx, owner).unwrap();
                assert_eq!(old.scope(), nested.scope());
                crate::exit_consumer::check(
                    tcx,
                    owner,
                    old.scopes().read_scope(),
                    old.exit(),
                    old.returning(),
                );
                assert_eq!(old.parameter(), proof.chains()[0].parameter());
                assert_eq!(old.bindings(), proof.chains()[0].bindings());
                assert_eq!(old.moves(), proof.chains()[0].moves());
                assert_eq!(old.drop_location(), proof.chains()[0].drop_location());
                assert_eq!(old.scalar_read(), proof.scalar_read().1);
                assert_eq!(
                    old.scopes().blocks().collect::<Vec<_>>(),
                    proof.scopes().blocks().collect::<Vec<_>>()
                );
                assert_eq!(old.scopes().bindings(), proof.scopes().bindings());
                assert_eq!(old.scopes().read_scope(), proof.scopes().read_scope());
                assert_eq!(old.scopes().drop_scope(), proof.chains()[0].drop_scope());
                assert_eq!(
                    old.into_construction().constructor(),
                    tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap()
                );
            } else {
                assert!(LinearOwnedBody::read_tail_scopes(tcx, owner).is_err());
            }
            let mut context = tcx;
            let binding = Builder::new().construction(Observe).build();
            for chain in proof.into_chains() {
                assert_eq!(
                    binding
                        .mapping()
                        .lower(&mut context, chain.into_construction())
                        .unwrap(),
                    tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap()
                );
            }
        } else {
            let error = match result {
                Err(error) => error,
                Ok(_) => panic!("admitted {name}"),
            };
            if name == "repeated_anchor" {
                assert_eq!(error, LinearError::Ambiguous);
            }
        }
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 14);
}
