//! Private corruption oracle; no malformed MIR crosses the query-only API.
use super::{super::super::flow, aggregate, relations, source};
use rustc_abi::{FieldIdx, VariantIdx};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};

pub(crate) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId) {
    let plan = source::read(tcx, owner).unwrap();
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let matched = relations::validate(tcx, owner, &plan, &original).unwrap();
    let trace = flow::trace(&original).unwrap();
    let aggregate = aggregate::read(tcx, &plan, &original, &trace).unwrap();
    assert_eq!(plan.record.fields().len(), 3);
    assert_eq!(matched.chains.len(), 3);
    assert_eq!(aggregate.fields.len(), 3);
    let first = FieldIdx::from_usize(0);
    let second = FieldIdx::from_usize(1);
    let other = tcx
        .hir_body_owners()
        .find(|id| tcx.def_path_str(*id) == "left::same")
        .unwrap();
    let other_plan = source::read(tcx, other).unwrap();
    let other_definition = other_plan.record.definition().did();
    let mut cases = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert!(
            relations::validate(tcx, owner, &plan, &changed).is_err(),
            "admitted field corruption: {name}"
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
    reject("parameter count", &|b| b.arg_count -= 1);
    reject("parameter type", &|b| {
        b.local_decls[mir::Local::from_usize(1)].ty = tcx.types.u32
    });
    reject("record local type", &|b| {
        b.local_decls[aggregate.destination.local].ty = other_plan.record.result()
    });
    reject("extra Box storage", &|b| {
        b.local_decls
            .push(b.local_decls[aggregate.fields[0].staging.local].clone());
    });
    reject("nominal definition", &|b| {
        let Rvalue::Aggregate(kind, _) = value(b, aggregate.location) else {
            unreachable!()
        };
        let mir::AggregateKind::Adt(def, ..) = &mut **kind else {
            unreachable!()
        };
        *def = other_definition;
    });
    reject("variant", &|b| {
        let Rvalue::Aggregate(kind, _) = value(b, aggregate.location) else {
            unreachable!()
        };
        let mir::AggregateKind::Adt(_, variant, ..) = &mut **kind else {
            unreachable!()
        };
        *variant = VariantIdx::from_usize(1);
    });
    reject("generic arguments", &|b| {
        let Rvalue::Aggregate(kind, _) = value(b, aggregate.location) else {
            unreachable!()
        };
        let mir::AggregateKind::Adt(_, _, args, ..) = &mut **kind else {
            unreachable!()
        };
        *args = tcx.mk_args(&[tcx.types.i32.into()]);
    });
    reject("active union field", &|b| {
        let Rvalue::Aggregate(kind, _) = value(b, aggregate.location) else {
            unreachable!()
        };
        let mir::AggregateKind::Adt(_, _, _, _, active) = &mut **kind else {
            unreachable!()
        };
        *active = Some(first);
    });
    reject("duplicate operand", &|b| {
        let Rvalue::Aggregate(_, operands) = value(b, aggregate.location) else {
            unreachable!()
        };
        operands[second] = operands[first].clone();
    });
    reject("swapped operands", &|b| {
        let Rvalue::Aggregate(_, operands) = value(b, aggregate.location) else {
            unreachable!()
        };
        let saved = operands[first].clone();
        operands[first] = operands[second].clone();
        operands[second] = saved;
    });
    reject("copied operand", &|b| {
        let Rvalue::Aggregate(_, operands) = value(b, aggregate.location) else {
            unreachable!()
        };
        let Operand::Move(place) = operands[first] else {
            unreachable!()
        };
        operands[first] = Operand::Copy(place);
    });
    reject("missing aggregate", &|b| {
        statement(b, aggregate.location).kind = StatementKind::Nop
    });
    reject("missing staging", &|b| {
        statement(b, aggregate.fields[0].movement).kind = StatementKind::Nop
    });
    reject("copied staging", &|b| {
        let Rvalue::Use(operand, _) = value(b, aggregate.fields[0].movement) else {
            unreachable!()
        };
        *operand = Operand::Copy(aggregate.fields[0].source);
    });
    reject("wrong staging origin", &|b| {
        let Rvalue::Use(operand, _) = value(b, aggregate.fields[0].movement) else {
            unreachable!()
        };
        *operand = Operand::Move(aggregate.fields[1].source);
    });
    reject("staging order", &|b| {
        let a = aggregate.fields[0].movement;
        let c = aggregate.fields[1].movement;
        assert_eq!(a.block, c.block);
        b.basic_blocks.as_mut()[a.block]
            .statements
            .swap(a.statement_index, c.statement_index);
    });
    let move_location = matched.chains[0].transfers.last().unwrap();
    let super::Transfer::Move { location: partial } = *move_location else {
        panic!("field extraction")
    };
    reject("missing partial move", &|b| {
        statement(b, partial).kind = StatementKind::Nop
    });
    reject("wrong partial field", &|b| {
        let Rvalue::Use(operand, _) = value(b, partial) else {
            unreachable!()
        };
        *operand = Operand::Move(aggregate.fields[1].place);
    });
    reject("wrong projection type", &|b| {
        let Rvalue::Use(operand, _) = value(b, partial) else {
            unreachable!()
        };
        *operand = Operand::Move(
            aggregate
                .destination
                .project_deeper(&[mir::ProjectionElem::Field(first, tcx.types.u32)], tcx),
        );
    });
    reject("copy partial move", &|b| {
        let Rvalue::Use(operand, _) = value(b, partial) else {
            unreachable!()
        };
        *operand = Operand::Copy(aggregate.fields[0].place);
    });
    reject("moved field cleanup", &|b| {
        let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
            [trace.drops[1].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *place = aggregate.fields[0].place;
    });
    reject("cleanup order", &|b| {
        for (index, next) in [(1, 2), (2, 1)] {
            let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
                [trace.drops[index].0.block]
                .terminator_mut()
                .kind
            else {
                unreachable!()
            };
            *place = trace.drops[next].1;
        }
    });
    reject("missing cleanup", &|b| {
        let term = b.basic_blocks.as_mut()[trace.drops[1].0.block].terminator_mut();
        let TerminatorKind::Drop { target, .. } = term.kind else {
            unreachable!()
        };
        term.kind = TerminatorKind::Goto { target };
    });
    reject("whole record cleanup", &|b| {
        let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()
            [trace.drops[1].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *place = aggregate.destination;
    });
    reject("duplicate constructor anchor", &|b| {
        let TerminatorKind::Call {
            args: original_args,
            ..
        } = trace.calls[0].1
        else {
            unreachable!()
        };
        let TerminatorKind::Call { args, .. } = &mut b.basic_blocks.as_mut()
            [trace.calls[1].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        args[0].node = original_args[0].node.clone();
    });
    reject("unaccounted assignment", &|b| {
        let copy = statement(b, aggregate.fields[0].movement).clone();
        b.basic_blocks.as_mut()[aggregate.location.block]
            .statements
            .push(copy);
    });
    reject("missing aggregate operand", &|b| {
        let Rvalue::Aggregate(_, operands) = value(b, aggregate.location) else {
            unreachable!()
        };
        operands.pop();
    });
    let cast = trace
        .assignments
        .iter()
        .find(|a| matches!(a.value, Rvalue::Cast(..)))
        .unwrap()
        .location;
    reject("wrong read owner", &|b| {
        let Rvalue::Cast(_, Operand::Copy(source), _) = value(b, cast) else {
            unreachable!()
        };
        source.local = matched.chains[1].places[0].local;
    });
    reject("unwind cleanup", &|b| {
        let TerminatorKind::Drop { unwind, .. } = &mut b.basic_blocks.as_mut()
            [trace.drops[0].0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *unwind = mir::UnwindAction::Continue;
    });
    assert_eq!(cases, 31);
    println!("31 record aggregate/place/cleanup corruptions rejected");
    let mut changed = source::read(tcx, owner).unwrap();
    let source::SourcePlace::Field { declaration, .. } = &mut changed.chains[0].places[1] else {
        panic!("source field")
    };
    *declaration = other_plan.record.fields()[0].declaration();
    assert!(relations::validate(tcx, owner, &changed, &original).is_err());
    let mut changed = source::read(tcx, owner).unwrap();
    let source::SourcePlace::Field { index, .. } = &mut changed.chains[0].places[1] else {
        panic!("source field")
    };
    *index = second;
    assert!(relations::validate(tcx, owner, &changed, &original).is_err());
    let mut changed = source::read(tcx, owner).unwrap();
    changed.record = other_plan.record;
    assert!(relations::validate(tcx, owner, &changed, &original).is_err());
    println!("3 source nominal/field substitutions rejected");
}

fn statement<'a, 'tcx>(
    body: &'a mut mir::Body<'tcx>,
    location: mir::Location,
) -> &'a mut mir::Statement<'tcx> {
    &mut body.basic_blocks.as_mut()[location.block].statements[location.statement_index]
}
fn value<'a, 'tcx>(body: &'a mut mir::Body<'tcx>, location: mir::Location) -> &'a mut Rvalue<'tcx> {
    let StatementKind::Assign(pair) = &mut statement(body, location).kind else {
        panic!("assignment")
    };
    &mut pair.1
}
