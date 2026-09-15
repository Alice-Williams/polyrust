//! Test-only mutation of cloned compiler MIR; never a safe construction route.
use super::super::flow;
use super::{BodyEnding, BodyStep, OwnedCallGraph, relations, source};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};

fn owner(tcx: TyCtxt<'_>, name: &str) -> LocalDefId {
    tcx.hir_body_owners()
        .find(|id| tcx.def_path_str(*id) == name)
        .unwrap()
}
pub(crate) fn check(tcx: TyCtxt<'_>) {
    let mut cases = 0;
    for name in [
        "source::via_producer",
        "source::via_producer_return",
        "source::via_consumer",
        "source::via_consumer_return",
        "source::via_relay",
        "source::via_relay_return",
        "producer_moved",
        "consumer_direct",
        "relay_direct",
    ] {
        let plan = source::entry(tcx, owner(tcx, name)).unwrap();
        let leaf = source::leaf(tcx, plan.call().unwrap()).unwrap();
        for plan in [plan, leaf] {
            cases += body_cases(tcx, &plan);
        }
    }
    for (a, b) in [
        ("source::via_producer", "source::via_producer_return"),
        ("source::via_consumer", "source::via_consumer_return"),
        ("source::via_relay", "source::via_relay_return"),
    ] {
        let a = OwnedCallGraph::read(tcx, owner(tcx, a)).unwrap();
        let b = OwnedCallGraph::read(tcx, owner(tcx, b)).unwrap();
        assert!(
            OwnedCallGraph::assemble(a.entry, b.leaf).is_err(),
            "same-signature callee substituted"
        );
        cases += 1;
    }
    let a = OwnedCallGraph::read(tcx, owner(tcx, "source::via_relay")).unwrap();
    let b = OwnedCallGraph::read(tcx, owner(tcx, "source::via_relay")).unwrap();
    assert!(
        OwnedCallGraph::assemble(a.entry, b.entry).is_err(),
        "entry used as leaf"
    );
    cases += 1;
    println!("owned-call corruption cases: {cases}");
    assert_eq!(cases, 475);
}
fn body_cases<'tcx>(tcx: TyCtxt<'tcx>, plan: &source::Plan<'tcx>) -> usize {
    let original = tcx
        .mir_drops_elaborated_and_const_checked(plan.frame.owner)
        .borrow();
    let matched = relations::validate(tcx, plan, &original).unwrap();
    let other = owner(tcx, "unproven");
    let mut cases = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert!(
            relations::validate(tcx, plan, &changed).is_err(),
            "admitted {name} in {}",
            tcx.def_path_str(plan.frame.owner)
        );
        cases += 1;
    };
    reject("owner", &|b| {
        b.source = tcx
            .mir_drops_elaborated_and_const_checked(other)
            .borrow()
            .source
    });
    reject("phase", &|b| b.phase = mir::MirPhase::Built);
    reject("parameter count", &|b| b.arg_count = 0);
    reject("parameter type", &|b| {
        b.local_decls[matched.parameter].ty = tcx.types.u32
    });
    reject("result type", &|b| {
        b.local_decls[mir::RETURN_PLACE].ty = tcx.types.u32
    });
    let boxed = original
        .local_decls
        .iter_enumerated()
        .find(|(_, d)| d.ty == plan.box_ty)
        .unwrap()
        .0;
    reject("unaccounted owner", &|b| {
        b.local_decls.push(b.local_decls[boxed].clone());
    });
    reject("unvisited block", &|b| {
        let data = b.basic_blocks[mir::START_BLOCK].clone();
        b.basic_blocks.as_mut().push(data);
    });
    reject("local budget", &|b| {
        while b.local_decls.len() <= 1024 {
            b.local_decls.push(b.local_decls[boxed].clone());
        }
    });
    reject("block budget", &|b| {
        let data = b.basic_blocks[mir::START_BLOCK].clone();
        while b.basic_blocks.len() <= 512 {
            b.basic_blocks.as_mut().push(data.clone());
        }
    });
    reject("missing return", &|b| {
        *term(b, matched.returning) = TerminatorKind::Unreachable
    });
    let mut assignments = Vec::new();
    for step in &matched.steps {
        match step {
            BodyStep::Move(at) => assignments.push(*at),
            BodyStep::Allocation(site) | BodyStep::LocalCall(site) => {
                assignments.extend(site.staging());
                let at = site.location();
                reject("missing argument", &|b| {
                    let TerminatorKind::Call { args, .. } = term(b, at) else {
                        unreachable!()
                    };
                    *args = Box::new([]);
                });
                reject("wrong argument", &|b| {
                    let TerminatorKind::Call { args, .. } = term(b, at) else {
                        unreachable!()
                    };
                    args[0].node = Operand::Move(mir::Place::from(site.destination()));
                });
                reject("missing call", &|b| {
                    let TerminatorKind::Call { target, .. } = term(b, at) else {
                        unreachable!()
                    };
                    let target = target.unwrap();
                    *term(b, at) = TerminatorKind::Goto { target };
                });
                reject("unwinding call", &|b| {
                    let TerminatorKind::Call { unwind, .. } = term(b, at) else {
                        unreachable!()
                    };
                    *unwind = mir::UnwindAction::Continue;
                });
                reject("same-type destination substitution", &|b| {
                    let new = b
                        .local_decls
                        .push(b.local_decls[site.destination()].clone());
                    let TerminatorKind::Call { destination, .. } = term(b, at) else {
                        unreachable!()
                    };
                    *destination = mir::Place::from(new);
                });
                if original.local_decls[site.argument()].ty == plan.box_ty {
                    reject("copied owned argument", &|b| {
                        let TerminatorKind::Call { args, .. } = term(b, at) else {
                            unreachable!()
                        };
                        args[0].node = Operand::Copy(mir::Place::from(site.argument()));
                    });
                } else {
                    reject("extra scalar staging edge", &|b| {
                        let fresh = b.local_decls.push(b.local_decls[site.argument()].clone());
                        let stage = site.staging()[0];
                        let mut statement =
                            b.basic_blocks[stage.block].statements[stage.statement_index].clone();
                        let StatementKind::Assign(pair) = &mut statement.kind else {
                            unreachable!()
                        };
                        pair.0 = mir::Place::from(fresh);
                        let Rvalue::Use(op, _) = &mut pair.1 else {
                            unreachable!()
                        };
                        *op = Operand::Copy(mir::Place::from(site.argument()));
                        b.basic_blocks.as_mut()[at.block].statements.push(statement);
                        let TerminatorKind::Call { args, .. } = term(b, at) else {
                            unreachable!()
                        };
                        args[0].node = Operand::Move(mir::Place::from(fresh));
                    });
                    reject("moved scalar staging", &|b| {
                        let Rvalue::Use(op, _) = value(b, site.staging()[0]) else {
                            unreachable!()
                        };
                        let Operand::Copy(place) = *op else {
                            unreachable!()
                        };
                        *op = Operand::Move(place);
                    });
                    reject("erased scalar staging", &|b| {
                        for location in site.staging() {
                            b.basic_blocks.as_mut()[location.block].statements
                                [location.statement_index]
                                .kind = StatementKind::Nop;
                        }
                        let TerminatorKind::Call { args, .. } = term(b, at) else {
                            unreachable!()
                        };
                        args[0].node = Operand::Copy(mir::Place::from(matched.parameter));
                    });
                }
                if let BodyStep::LocalCall(_) = step {
                    let role = plan.call().unwrap().role();
                    use crate::owned_source::local_call::CallRole;
                    let alternate = match role {
                        CallRole::Producer => "source::via_producer_return",
                        CallRole::Consumer => "consumer_direct",
                        CallRole::Relay => "relay_direct",
                    };
                    let alternate = if owner(tcx, alternate) == plan.frame.owner {
                        match role {
                            CallRole::Producer => "source::via_producer",
                            CallRole::Consumer => "source::via_consumer",
                            CallRole::Relay => "source::via_relay",
                        }
                    } else {
                        alternate
                    };
                    let alt_plan = source::entry(tcx, owner(tcx, alternate)).unwrap();
                    assert_ne!(
                        alt_plan.call().unwrap().callee(),
                        plan.call().unwrap().callee()
                    );
                    let alt_body = tcx
                        .mir_drops_elaborated_and_const_checked(alt_plan.frame.owner)
                        .borrow();
                    let alt_trace = flow::trace(&alt_body).unwrap();
                    let TerminatorKind::Call { func, .. } = alt_trace.calls.last().unwrap().1
                    else {
                        unreachable!()
                    };
                    reject("same-signature callee", &|b| {
                        let TerminatorKind::Call { func: original, .. } = term(b, at) else {
                            unreachable!()
                        };
                        *original = func.clone();
                    });
                }
            }
        }
    }
    match matched.ending {
        BodyEnding::ReadDrop { cast, read, drop } => {
            assignments.extend([cast, read]);
            reject("wrong drop", &|b| {
                let TerminatorKind::Drop { place, .. } = term(b, drop) else {
                    unreachable!()
                };
                *place = mir::Place::from(mir::RETURN_PLACE);
            });
            reject("missing drop", &|b| {
                let TerminatorKind::Drop { target, .. } = term(b, drop) else {
                    unreachable!()
                };
                let target = *target;
                *term(b, drop) = TerminatorKind::Goto { target };
            });
            reject("read wrong owner", &|b| {
                let Rvalue::Cast(_, Operand::Copy(place), _) = value(b, cast) else {
                    unreachable!()
                };
                place.local = mir::RETURN_PLACE;
            });
        }
        BodyEnding::OwnerReturn(at) => assignments.push(at),
        BodyEnding::DirectReturn => {}
    }
    assignments.sort_by_key(|at| (at.block.as_usize(), at.statement_index));
    assignments.dedup();
    for at in assignments {
        reject("missing transfer/read", &|b| {
            b.basic_blocks.as_mut()[at.block].statements[at.statement_index].kind =
                StatementKind::Nop
        });
        reject("duplicate transfer/read", &|b| {
            let statement = b.basic_blocks[at.block].statements[at.statement_index].clone();
            b.basic_blocks.as_mut()[at.block].statements.push(statement);
        });
        if matches!(value(&mut original.clone(), at), Rvalue::Use(Operand::Move(p), _) if original.local_decls[p.local].ty == plan.box_ty)
        {
            reject("copied owner edge", &|b| {
                let Rvalue::Use(op, _) = value(b, at) else {
                    unreachable!()
                };
                let Operand::Move(place) = *op else {
                    unreachable!()
                };
                *op = Operand::Copy(place);
            });
        }
    }
    cases
}
fn term<'a, 'tcx>(
    body: &'a mut mir::Body<'tcx>,
    at: mir::Location,
) -> &'a mut TerminatorKind<'tcx> {
    &mut body.basic_blocks.as_mut()[at.block].terminator_mut().kind
}
fn value<'a, 'tcx>(body: &'a mut mir::Body<'tcx>, at: mir::Location) -> &'a mut Rvalue<'tcx> {
    let StatementKind::Assign(pair) =
        &mut body.basic_blocks.as_mut()[at.block].statements[at.statement_index].kind
    else {
        unreachable!()
    };
    &mut pair.1
}
