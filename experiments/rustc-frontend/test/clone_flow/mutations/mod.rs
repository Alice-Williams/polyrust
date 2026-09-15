//! Private rejection oracles over cloned MIR, never an input API for backends.
mod loans;
mod provenance;
mod residual;
use super::super::{
    cloning::{relations, source},
    flow,
};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};
type Reject<'a, 'tcx> = dyn FnMut(&str, &dyn Fn(&mut mir::Body<'tcx>)) + 'a;

pub(crate) fn check(tcx: TyCtxt<'_>) {
    let mut cases = 0;
    for name in [
        "method",
        "qualified",
        "read_original",
        "return_original",
        "return_clone",
        "moves_before",
        "moves_after",
        "interleaved",
        "shadow_original",
        "shadow_clone",
    ] {
        let owner = owner(tcx, name);
        let plan = source::read(tcx, owner).unwrap();
        cases += body_cases(tcx, &plan);
    }
    println!("clone-flow corruption cases: {cases}");
    assert_eq!(cases, 528);
}
fn owner(tcx: TyCtxt<'_>, name: &str) -> LocalDefId {
    tcx.hir_body_owners()
        .find(|id| tcx.def_path_str(*id) == name)
        .unwrap()
}
fn body_cases<'tcx>(tcx: TyCtxt<'tcx>, plan: &source::Plan<'tcx>) -> usize {
    let original = tcx
        .mir_drops_elaborated_and_const_checked(plan.frame.owner)
        .borrow();
    let matched = relations::validate(tcx, plan, &original).unwrap();
    let trace = flow::trace(&original).unwrap();
    let mut cases = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert!(
            relations::validate(tcx, plan, &changed).is_err(),
            "accepted {name} in {}",
            tcx.def_path_str(plan.frame.owner)
        );
        cases += 1;
    };
    let other = if tcx.def_path_str(plan.frame.owner) == "method" {
        "qualified"
    } else {
        "method"
    };
    reject("owner", &|b| {
        b.source = tcx
            .mir_drops_elaborated_and_const_checked(owner(tcx, other))
            .borrow()
            .source
    });
    reject("phase", &|b| b.phase = mir::MirPhase::Built);
    reject("argument count", &|b| b.arg_count = 0);
    reject("argument type", &|b| {
        b.local_decls[matched.parameter].ty = tcx.types.u32
    });
    reject("return type", &|b| {
        b.local_decls[mir::RETURN_PLACE].ty = tcx.types.u32
    });
    let first_owner = matched.bindings[0].2;
    reject("extra owner", &|b| {
        b.local_decls.push(b.local_decls[first_owner].clone());
    });
    reject("unvisited block", &|b| {
        let block = b.basic_blocks[mir::START_BLOCK].clone();
        b.basic_blocks.as_mut().push(block);
    });
    reject("local budget", &|b| {
        while b.local_decls.len() <= 1024 {
            b.local_decls.push(b.local_decls[first_owner].clone());
        }
    });
    reject("block budget", &|b| {
        let block = b.basic_blocks[mir::START_BLOCK].clone();
        while b.basic_blocks.len() <= 512 {
            b.basic_blocks.as_mut().push(block.clone());
        }
    });
    reject("missing return", &|b| {
        *term(b, matched.returning) = TerminatorKind::Unreachable
    });
    for &(at, _) in &trace.calls {
        reject("missing call", &|b| {
            let TerminatorKind::Call { target, .. } = term(b, at) else {
                unreachable!()
            };
            let target = target.unwrap();
            *term(b, at) = TerminatorKind::Goto { target };
        });
        reject("missing argument", &|b| {
            let TerminatorKind::Call { args, .. } = term(b, at) else {
                unreachable!()
            };
            *args = Box::new([]);
        });
        reject("unwinding call", &|b| {
            let TerminatorKind::Call { unwind, .. } = term(b, at) else {
                unreachable!()
            };
            *unwind = mir::UnwindAction::Continue;
        });
        reject("argument uses owner", &|b| {
            let TerminatorKind::Call { args, .. } = term(b, at) else {
                unreachable!()
            };
            args[0].node = Operand::Move(mir::Place::from(first_owner));
        });
        reject("fresh disconnected call destination", &|b| {
            let fresh = b.local_decls.push(b.local_decls[first_owner].clone());
            let TerminatorKind::Call { destination, .. } = term(b, at) else {
                unreachable!()
            };
            *destination = mir::Place::from(fresh);
        });
    }
    let cloning_at = trace.calls[1].0;
    reject("aliased clone destination", &|b| {
        let TerminatorKind::Call { destination, .. } = term(b, cloning_at) else {
            unreachable!()
        };
        *destination = mir::Place::from(first_owner);
    });
    reject("constructor substituted for clone", &|b| {
        let TerminatorKind::Call {
            func: constructor, ..
        } = trace.calls[0].1
        else {
            unreachable!()
        };
        let TerminatorKind::Call { func, .. } = term(b, cloning_at) else {
            unreachable!()
        };
        *func = constructor.clone();
    });
    for assignment in &trace.assignments {
        if original.local_decls[assignment.destination.local].ty == tcx.types.bool {
            continue;
        }
        let at = assignment.location;
        reject("missing assignment", &|b| {
            b.basic_blocks.as_mut()[at.block].statements[at.statement_index].kind =
                StatementKind::Nop
        });
        reject("duplicate assignment", &|b| {
            let statement = b.basic_blocks[at.block].statements[at.statement_index].clone();
            b.basic_blocks.as_mut()[at.block].statements.push(statement);
        });
        if let Rvalue::Use(Operand::Move(source), _) = assignment.value {
            let source = *source;
            reject("copy replaces owner move", &|b| {
                let Rvalue::Use(op, _) = value(b, at) else {
                    unreachable!()
                };
                *op = Operand::Copy(source);
            });
        }
    }
    for &(_, place, at) in &matched.drops {
        reject("missing cleanup", &|b| {
            let TerminatorKind::Drop { target, .. } = term(b, at) else {
                unreachable!()
            };
            let target = *target;
            *term(b, at) = TerminatorKind::Goto { target };
        });
        let other = matched
            .drops
            .iter()
            .find(|(_, p, _)| *p != place)
            .unwrap()
            .1;
        reject("same-type other cleanup", &|b| {
            let TerminatorKind::Drop { place, .. } = term(b, at) else {
                unreachable!()
            };
            *place = other;
        });
        reject("unwinding cleanup", &|b| {
            let TerminatorKind::Drop { unwind, .. } = term(b, at) else {
                unreachable!()
            };
            *unwind = mir::UnwindAction::Continue;
        });
    }
    reject("reordered cleanup", &|b| {
        for index in 0..2 {
            let TerminatorKind::Drop { place, .. } = term(b, matched.drops[index].2) else {
                unreachable!()
            };
            *place = matched.drops[1 - index].1;
        }
    });
    loans::cases(tcx, plan, &original, &matched, &mut reject);
    provenance::cases(tcx, &original, &matched, &mut reject);
    let isolated = residual::cases(tcx, plan, &original, &mut reject);
    cases + isolated
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
        panic!("assignment")
    };
    &mut pair.1
}
