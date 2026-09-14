//! Private mutated MIR cannot enter the exposed compiler-query constructor.
use super::{super::flow, relations, residual, source};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};
use std::collections::HashSet;

pub(crate) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId) {
    let plan = source::read(tcx, owner).unwrap();
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let matched = relations::validate(tcx, owner, &plan, &original).unwrap();
    let trace = flow::trace(&original).unwrap();
    assert_eq!(trace.calls.len(), 2);
    assert_eq!(trace.drops.len(), 2);
    assert_eq!(matched.chains[0].owners.len(), 3);
    assert_eq!(matched.chains[1].owners.len(), 2);
    let move_location = matched.chains[0].moves[0];
    let cast = trace
        .assignments
        .iter()
        .find(|a| matches!(a.value, Rvalue::Cast(..)))
        .unwrap()
        .location;
    let bool_writes: Vec<_> = trace
        .assignments
        .iter()
        .filter(|a| original.local_decls[a.destination.local].ty == tcx.types.bool)
        .collect();
    assert_eq!(bool_writes.len(), 4);
    let bool_local = bool_writes[0].destination.local;
    let bool_write = bool_writes[0].location;
    let bool_value = |index: usize| {
        let Rvalue::Use(Operand::Constant(value), _) = bool_writes[index].value else {
            panic!("Boolean constant")
        };
        value
            .const_
            .try_eval_bool(tcx, rustc_middle::ty::TypingEnv::fully_monomorphized())
            .unwrap()
    };
    assert!(!bool_value(0));
    assert!(bool_value(1));
    let mut cases = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert!(
            relations::validate(tcx, owner, &plan, &changed).is_err(),
            "multi-owner mutation admitted: {name}"
        );
        cases += 1;
    };
    reject("wrong owner", &|b| {
        let other = tcx
            .hir_body_owners()
            .find(|id| tcx.def_path_str(*id) == "one")
            .unwrap();
        b.source = tcx
            .mir_drops_elaborated_and_const_checked(other)
            .borrow()
            .source;
    });
    reject("wrong signature", &|b| b.arg_count = 1);
    reject("wrong phase", &|b| b.phase = mir::MirPhase::Built);
    let arguments: Vec<_> = trace
        .calls
        .iter()
        .map(|(_, call)| {
            let TerminatorKind::Call { args, .. } = call else {
                unreachable!()
            };
            args[0].node.clone()
        })
        .collect();
    reject("swapped argument anchors", &|b| {
        for (index, &(location, _)) in trace.calls.iter().enumerate() {
            let TerminatorKind::Call { args, .. } = &mut b.basic_blocks.as_mut()[location.block]
                .terminator_mut()
                .kind
            else {
                unreachable!()
            };
            args[0].node = arguments[1 - index].clone();
        }
    });
    reject("duplicate anchor", &|b| {
        let TerminatorKind::Call { args, .. } = &mut b.basic_blocks.as_mut()
            [trace.calls[1].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        args[0].node = arguments[0].clone();
    });
    reject("wrong constructor destination", &|b| {
        let TerminatorKind::Call { destination, .. } = &mut b.basic_blocks.as_mut()
            [trace.calls[0].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *destination = mir::Place::from(matched.chains[1].owners[0]);
    });
    reject("missing constructor", &|b| {
        let term = b.basic_blocks.as_mut()[trace.calls[0].0.block].terminator_mut();
        let TerminatorKind::Call {
            target: Some(target),
            ..
        } = term.kind
        else {
            unreachable!()
        };
        term.kind = TerminatorKind::Goto { target };
    });
    reject("missing move", &|b| {
        statement(b, move_location).kind = StatementKind::Nop
    });
    reject("duplicate move", &|b| {
        let copy = statement(b, move_location).clone();
        b.basic_blocks.as_mut()[move_location.block]
            .statements
            .insert(move_location.statement_index, copy);
    });
    reject("cross-chain move", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, move_location).kind else {
            unreachable!()
        };
        let Rvalue::Use(operand, _) = &mut pair.1 else {
            unreachable!()
        };
        *operand = Operand::Move(mir::Place::from(matched.chains[1].owners[0]));
    });
    reject("self move", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, move_location).kind else {
            unreachable!()
        };
        let Rvalue::Use(operand, _) = &mut pair.1 else {
            unreachable!()
        };
        *operand = Operand::Move(pair.0);
    });
    reject("copy instead of move", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, move_location).kind else {
            unreachable!()
        };
        let Rvalue::Use(operand, _) = &mut pair.1 else {
            unreachable!()
        };
        *operand = Operand::Copy(mir::Place::from(matched.chains[0].owners[0]));
    });
    reject("read other live owner", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, cast).kind else {
            unreachable!()
        };
        let Rvalue::Cast(_, Operand::Copy(place), _) = &mut pair.1 else {
            unreachable!()
        };
        place.local = *matched.chains[0].owners.last().unwrap();
    });
    reject("swapped cleanup order", &|b| {
        for (index, &(location, _)) in trace.drops.iter().enumerate() {
            let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()[location.block]
                .terminator_mut()
                .kind
            else {
                unreachable!()
            };
            *place = trace.drops[1 - index].1;
        }
    });
    reject("duplicate owner cleanup", &|b| {
        let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
            [trace.drops[1].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *place = trace.drops[0].1;
    });
    reject("drop moved source", &|b| {
        let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
            [trace.drops[0].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *place = mir::Place::from(matched.chains[0].owners[0]);
    });
    reject("async cleanup successor", &|b| {
        let TerminatorKind::Drop { target, drop, .. } = &mut b.basic_blocks.as_mut()
            [trace.drops[0].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        assert!(drop.is_none());
        *drop = Some(*target);
    });
    reject("missing cleanup", &|b| {
        let term = b.basic_blocks.as_mut()[trace.drops[0].0.block].terminator_mut();
        let TerminatorKind::Drop { target, .. } = term.kind else {
            unreachable!()
        };
        term.kind = TerminatorKind::Goto { target };
    });
    reject("extra owner", &|b| {
        b.local_decls
            .push(b.local_decls[matched.chains[0].owners[0]].clone());
    });
    reject("unreachable block", &|b| {
        let block = b.basic_blocks[trace.returning.block].clone();
        b.basic_blocks.as_mut().push(block);
    });
    reject("nonconstant bookkeeping", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, bool_write).kind else {
            unreachable!()
        };
        let Rvalue::Use(operand, _) = &mut pair.1 else {
            unreachable!()
        };
        *operand = Operand::Copy(mir::Place::from(bool_local));
    });
    // Isolate the local-use visitor: all non-Boolean assignments are marked
    // already consumed here, so a new Boolean read must fail that visitor.
    let mut observed = original.clone();
    let TerminatorKind::Call { args, .. } = &mut observed.basic_blocks.as_mut()
        [trace.calls[0].0.block]
        .terminator_mut()
        .kind
    else {
        unreachable!()
    };
    args[0].node = Operand::Copy(mir::Place::from(bool_local));
    let observed_trace = flow::trace(&observed).unwrap();
    let mut used: HashSet<_> = observed_trace
        .assignments
        .iter()
        .filter(|a| observed.local_decls[a.destination.local].ty != tcx.types.bool)
        .map(|a| a.location)
        .collect();
    assert!(residual::account(tcx, &observed, &observed_trace, &mut used).is_err());
    // Unread constant values are not secretly being used as owner identities.
    let mut unobserved = original.clone();
    let replacement = bool_writes[1].value.clone();
    let StatementKind::Assign(pair) = &mut statement(&mut unobserved, bool_write).kind else {
        unreachable!()
    };
    pair.1 = replacement;
    assert!(relations::validate(tcx, owner, &plan, &unobserved).is_ok());
    assert_eq!(cases, 21);
    println!("22 multi-owner corruptions rejected; unread constant substitution remains harmless");
}
fn statement<'a, 'tcx>(
    body: &'a mut mir::Body<'tcx>,
    location: mir::Location,
) -> &'a mut mir::Statement<'tcx> {
    &mut body.basic_blocks.as_mut()[location.block].statements[location.statement_index]
}
