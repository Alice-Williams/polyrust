//! Private test-only corrupted MIR is never an input to public admission.
use super::{flow, relations, source};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};

pub(crate) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId) {
    let plan = source::read(tcx, owner).expect("canonical plan");
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let matched = relations::validate(tcx, &plan, &original).expect("unmodified evidence");
    let flow = flow::read(&original).unwrap();
    let first = matched.owners[0];
    let last = *matched.owners.last().unwrap();
    let moves: Vec<_> = flow.assignments.iter().filter(|a|
        matches!(a.value, Rvalue::Use(Operand::Move(p), _) if matched.owners.contains(&p.local))
    ).map(|a| a.location).collect();
    assert_eq!(moves.len(), 3);
    let cast = flow
        .assignments
        .iter()
        .find(|a| matches!(a.value, Rvalue::Cast(..)))
        .unwrap()
        .location;
    let scalar = flow.assignments.iter().find(|a|
        matches!(a.value, Rvalue::Use(Operand::Copy(p), _) if p.local == matched.parameter && p.projection.is_empty())
    ).unwrap().location;
    let mut cases = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert!(
            relations::validate(tcx, &plan, &changed).is_err(),
            "mutation admitted: {name}"
        );
        cases += 1;
    };
    reject("wrong body owner", &|b| {
        let other = tcx
            .hir_body_owners()
            .find(|id| tcx.def_path_str(*id) == "zero")
            .unwrap();
        b.source = tcx
            .mir_drops_elaborated_and_const_checked(other)
            .borrow()
            .source;
    });
    reject("wrong phase", &|b| b.phase = mir::MirPhase::Built);
    reject("wrong parameter count", &|b| b.arg_count = 0);
    reject("different argument producer", &|b| {
        let TerminatorKind::Call { args, .. } = &mut b.basic_blocks.as_mut()[flow.call.0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        args[0].node = Operand::Copy(mir::Place::from(mir::RETURN_PLACE));
    });
    reject("wrong construction destination", &|b| {
        let TerminatorKind::Call { destination, .. } = &mut b.basic_blocks.as_mut()
            [flow.call.0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *destination = mir::Place::from(last);
    });
    reject("missing move", &|b| {
        statement(b, moves[0]).kind = StatementKind::Nop
    });
    reject("duplicate move", &|b| {
        let copy = statement(b, moves[0]).clone();
        b.basic_blocks.as_mut()[moves[0].block]
            .statements
            .insert(moves[0].statement_index, copy);
    });
    reject("copy instead of move", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, moves[0]).kind else {
            unreachable!()
        };
        let Rvalue::Use(operand, _) = &mut pair.1 else {
            unreachable!()
        };
        *operand = Operand::Copy(mir::Place::from(first));
    });
    reject("self move", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, moves[0]).kind else {
            unreachable!()
        };
        let Rvalue::Use(operand, _) = &mut pair.1 else {
            unreachable!()
        };
        *operand = Operand::Move(pair.0);
    });
    reject("cycle destination", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, moves[2]).kind else {
            unreachable!()
        };
        pair.0 = mir::Place::from(first);
    });
    reject("read moved owner", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, cast).kind else {
            unreachable!()
        };
        let Rvalue::Cast(_, Operand::Copy(p), _) = &mut pair.1 else {
            unreachable!()
        };
        p.local = first;
    });
    reject("wrong read producer", &|b| {
        let StatementKind::Assign(pair) = &mut statement(b, matched.scalar_read).kind else {
            unreachable!()
        };
        let Rvalue::Use(operand, _) = &mut pair.1 else {
            unreachable!()
        };
        *operand = Operand::Copy(mir::Place::from(matched.parameter));
    });
    reject("drop moved owner", &|b| {
        let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()[flow.drop.0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *place = mir::Place::from(first);
    });
    reject("missing drop", &|b| {
        let term = b.basic_blocks.as_mut()[flow.drop.0.block].terminator_mut();
        let TerminatorKind::Drop { target, .. } = term.kind else {
            unreachable!()
        };
        term.kind = TerminatorKind::Goto { target };
    });
    reject("extra box local", &|b| {
        b.local_decls.push(b.local_decls[first].clone());
    });
    reject("unreachable block", &|b| {
        let block = b.basic_blocks[flow.returning.block].clone();
        b.basic_blocks.as_mut().push(block);
    });
    reject("control-flow cycle", &|b| {
        b.basic_blocks.as_mut()[flow.returning.block]
            .terminator_mut()
            .kind = TerminatorKind::Goto {
            target: mir::START_BLOCK,
        };
    });
    reject("cleanup block", &|b| {
        b.basic_blocks.as_mut()[flow.drop.0.block].is_cleanup = true
    });
    reject("extra call", &|b| {
        let call = b.basic_blocks[flow.call.0.block].terminator().kind.clone();
        b.basic_blocks.as_mut()[flow.returning.block]
            .terminator_mut()
            .kind = call;
    });
    reject("extra drop", &|b| {
        let drop = b.basic_blocks[flow.drop.0.block].terminator().kind.clone();
        b.basic_blocks.as_mut()[flow.returning.block]
            .terminator_mut()
            .kind = drop;
    });
    reject("duplicate scalar definition", &|b| {
        let copy = statement(b, scalar).clone();
        b.basic_blocks.as_mut()[scalar.block]
            .statements
            .insert(scalar.statement_index, copy);
    });
    reject("missing argument definition", &|b| {
        statement(b, scalar).kind = StatementKind::Nop
    });
    let mut missing = source::read(tcx, owner).unwrap();
    missing.bindings.pop();
    assert!(relations::validate(tcx, &missing, &original).is_err());
    let mut duplicate = source::read(tcx, owner).unwrap();
    duplicate.bindings.push(duplicate.bindings[0]);
    assert!(relations::validate(tcx, &duplicate, &original).is_err());
    assert_eq!(cases, 22);
    println!("24 private ownership correspondence mutations rejected");
}

fn statement<'a, 'tcx>(
    body: &'a mut mir::Body<'tcx>,
    location: mir::Location,
) -> &'a mut mir::Statement<'tcx> {
    &mut body.basic_blocks.as_mut()[location.block].statements[location.statement_index]
}
