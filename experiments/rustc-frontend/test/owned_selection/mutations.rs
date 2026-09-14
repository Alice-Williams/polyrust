//! Wrong flag decisions cannot justify wrong source transfers or cleanup.
use super::super::super::{exits, flow, scopes};
use super::{SelectedOwnedBody, source};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::{self, TyCtxt},
};

pub(crate) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId) {
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let proof = SelectedOwnedBody::from_body(tcx, owner, &original).unwrap();
    let [no, yes] = flow::selection(tcx, &original).unwrap();
    let boolean = |wanted| {
        original
            .basic_blocks
            .iter()
            .flat_map(|b| &b.statements)
            .find_map(|s| {
                let StatementKind::Assign(pair) = &s.kind else {
                    return None;
                };
                let Rvalue::Use(Operand::Constant(c), _) = &pair.1 else {
                    return None;
                };
                (c.const_.ty() == tcx.types.bool
                    && c.const_
                        .try_eval_bool(tcx, ty::TypingEnv::fully_monomorphized())
                        == Some(wanted))
                .then(|| c.clone())
            })
            .unwrap()
    };
    let mut count = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert!(
            SelectedOwnedBody::from_body(tcx, owner, &changed).is_err(),
            "admitted selection corruption {name}"
        );
        count += 1;
    };
    reject("phase", &|b| b.phase = mir::MirPhase::Built);
    reject("signature", &|b| b.arg_count -= 1);
    reject("owner", &|b| {
        let other = tcx
            .hir_body_owners()
            .find(|id| tcx.def_path_str(*id) == "select")
            .unwrap();
        b.source = tcx
            .mir_drops_elaborated_and_const_checked(other)
            .borrow()
            .source;
    });
    reject("constant guard", &|b| {
        let TerminatorKind::SwitchInt { discr, .. } = &mut b.basic_blocks.as_mut()
            [proof.guard().block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *discr = Operand::Constant(boolean(true));
    });
    reject("swapped user successors", &|b| {
        swap_edges(b, proof.guard().block)
    });
    for (index, trace) in [&no, &yes].into_iter().enumerate() {
        let path = &proof.paths()[index];
        let chosen = path
            .chains()
            .iter()
            .find(|c| c.bindings().last().unwrap().0 == proof.binding())
            .unwrap();
        let transfer = *chosen.moves().last().unwrap();
        let before = chosen.bindings()[chosen.bindings().len() - 2].1;
        for decision in &trace.cleanup {
            reject("flipped reaching flag", &|b| {
                let (_, value) = assignment(b, decision.definition());
                let Rvalue::Use(operand, _) = value else {
                    unreachable!()
                };
                *operand = Operand::Constant(boolean(!decision.value()));
            });
            reject("omitted reaching flag", &|b| {
                b.basic_blocks.as_mut()[decision.definition().block].statements
                    [decision.definition().statement_index]
                    .kind = StatementKind::Nop;
            });
        }
        reject("copied owner", &|b| {
            let (_, value) = assignment(b, transfer);
            let Rvalue::Use(operand, _) = value else {
                unreachable!()
            };
            *operand = Operand::Copy(mir::Place::from(before));
        });
        reject("different branch destination", &|b| {
            let selected = chosen.bindings().last().unwrap().1;
            let replacement = b.local_decls.push(b.local_decls[selected].clone());
            assignment(b, transfer).0 = mir::Place::from(replacement);
        });
        reject("read moved-from owner", &|b| {
            let cast = trace
                .assignments
                .iter()
                .find(|a| matches!(a.value, Rvalue::Cast(..)))
                .unwrap()
                .location;
            let (_, value) = assignment(b, cast);
            let Rvalue::Cast(_, Operand::Copy(place), _) = value else {
                unreachable!()
            };
            place.local = before;
        });
        reject("missing cleanup", &|b| {
            let term = b.basic_blocks.as_mut()[trace.drops[1].0.block].terminator_mut();
            let TerminatorKind::Drop { target, .. } = term.kind else {
                unreachable!()
            };
            term.kind = TerminatorKind::Goto { target };
        });
        reject("duplicate cleanup", &|b| {
            let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
                [trace.drops[1].0.block]
                .terminator_mut()
                .kind
            else {
                unreachable!()
            };
            *place = trace.drops[0].1;
        });
        reject("cleanup order", &|b| {
            for index in 0..2 {
                let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
                    [trace.drops[index].0.block]
                    .terminator_mut()
                    .kind
                else {
                    unreachable!()
                };
                *place = trace.drops[1 - index].1;
            }
        });
    }
    for decision in &no.cleanup {
        reject("swapped cleanup successors", &|b| {
            swap_edges(b, decision.location().block)
        });
        reject("constant cleanup operand", &|b| {
            let TerminatorKind::SwitchInt { discr, .. } = &mut b.basic_blocks.as_mut()
                [decision.location().block]
                .terminator_mut()
                .kind
            else {
                unreachable!()
            };
            *discr = Operand::Constant(boolean(decision.value()));
        });
    }
    reject("extra unreachable block", &|b| {
        let extra = b.basic_blocks[no.returning.block].clone();
        b.basic_blocks.as_mut().push(extra);
    });
    reject("async cleanup", &|b| {
        let TerminatorKind::Drop { drop, target, .. } = &mut b.basic_blocks.as_mut()
            [no.drops[0].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *drop = Some(*target);
    });
    reject("unwind", &|b| {
        let TerminatorKind::Drop { unwind, .. } = &mut b.basic_blocks.as_mut()[no.drops[0].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *unwind = mir::UnwindAction::Continue;
    });
    reject("missing shared return", &|b| {
        b.basic_blocks.as_mut()[no.returning.block]
            .terminator_mut()
            .kind = TerminatorKind::Goto {
            target: mir::START_BLOCK,
        }
    });
    assert_eq!(count, 33);
    flag_uses(tcx, owner, &original, &no);
    source_claims(tcx, owner);
    println!("33 selection MIR, 2 isolated flag and 4 source corruptions rejected");
}

fn assignment<'a, 'tcx>(
    body: &'a mut mir::Body<'tcx>,
    location: mir::Location,
) -> &'a mut (mir::Place<'tcx>, Rvalue<'tcx>) {
    let StatementKind::Assign(pair) =
        &mut body.basic_blocks.as_mut()[location.block].statements[location.statement_index].kind
    else {
        panic!("assignment")
    };
    pair
}
fn swap_edges(body: &mut mir::Body<'_>, block: mir::BasicBlock) {
    let TerminatorKind::SwitchInt { targets, .. } =
        &mut body.basic_blocks.as_mut()[block].terminator_mut().kind
    else {
        panic!("switch")
    };
    *targets = mir::SwitchTargets::new(
        [(0, targets.target_for_value(1))].into_iter(),
        targets.target_for_value(0),
    );
}

fn flag_uses<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    body: &mir::Body<'tcx>,
    no: &flow::Trace<'_, 'tcx>,
) {
    let decision = &no.cleanup[0];
    let mut observed = body.clone();
    let temporary = observed
        .local_decls
        .push(observed.local_decls[decision.local()].clone());
    let mut statement = body.basic_blocks[decision.definition().block].statements
        [decision.definition().statement_index]
        .clone();
    let StatementKind::Assign(pair) = &mut statement.kind else {
        unreachable!()
    };
    pair.0 = mir::Place::from(temporary);
    let Rvalue::Use(operand, _) = &mut pair.1 else {
        unreachable!()
    };
    *operand = Operand::Copy(mir::Place::from(decision.local()));
    observed.basic_blocks.as_mut()[no.returning.block]
        .statements
        .push(statement);
    assert!(
        flow::selection(tcx, &observed).is_err(),
        "unaccounted flag read"
    );
    assert!(SelectedOwnedBody::from_body(tcx, owner, &observed).is_err());

    let mut changed = body.clone();
    let parameter = SelectedOwnedBody::from_body(tcx, owner, body)
        .unwrap()
        .parameter()
        .1;
    let (_, value) = assignment(&mut changed, decision.definition());
    let Rvalue::Use(operand, _) = value else {
        unreachable!()
    };
    *operand = Operand::Copy(mir::Place::from(parameter));
    assert!(
        flow::selection(tcx, &changed).is_err(),
        "nonconstant flag definition"
    );
    assert!(SelectedOwnedBody::from_body(tcx, owner, &changed).is_err());
}

fn source_claims(tcx: TyCtxt<'_>, owner: LocalDefId) {
    let shape = source::read(tcx, owner).unwrap();
    assert!(
        source::operand(
            tcx,
            owner,
            shape.operands[0],
            shape.binding,
            exits::Outcome::False
        )
        .is_err()
    );
    assert!(
        source::operand(
            tcx,
            owner,
            shape.branch,
            shape.parameter,
            exits::Outcome::False
        )
        .is_err()
    );
    let plan = super::super::source::read_selection(tcx, owner, exits::Outcome::False).unwrap();
    let fresh = || scopes::ContainmentClaims {
        blocks: plan.scopes.blocks().collect(),
        bindings: plan.scopes.bindings().to_vec(),
        read: plan.scopes.read_scope(),
    };
    let mut wrong_scope = fresh();
    wrong_scope.read = shape.binding;
    assert!(
        scopes::certify_route(
            tcx,
            owner,
            wrong_scope,
            plan.scopes.exit(),
            exits::Mode::Selection(exits::Outcome::False)
        )
        .is_err()
    );
    assert!(
        scopes::certify_route(
            tcx,
            owner,
            fresh(),
            exits::Exit::Tail(shape.branch),
            exits::Mode::Selection(exits::Outcome::False)
        )
        .is_err()
    );
}
