//! Deliberate private corruption of authentic compiler data, never public input.
use super::{super::super::flow, relations, source};
use rustc_abi::{FieldIdx, VariantIdx};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};

pub(crate) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId) {
    let plan = source::read(tcx, owner).unwrap();
    let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let matched = relations::validate(tcx, &plan, &original).unwrap();
    let trace = flow::read(&original).unwrap();
    let other = tcx
        .hir_body_owners()
        .find(|id| tcx.def_path_str(*id) == "left::same")
        .unwrap();
    let other_plan = source::read(tcx, other).unwrap();
    let first = FieldIdx::from_usize(0);
    let second = FieldIdx::from_usize(1);
    let mut cases = 0;
    let mut reject = |name: &str, edit: &dyn Fn(&mut mir::Body<'tcx>)| {
        let mut changed = original.clone();
        edit(&mut changed);
        assert!(
            relations::validate(tcx, &plan, &changed).is_err(),
            "admitted corruption: {name}"
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
    reject("result type", &|b| {
        b.local_decls[mir::RETURN_PLACE].ty = tcx.types.bool
    });
    reject("record type", &|b| {
        b.local_decls[matched.payload.record].ty = other_plan.constructor.payload().ty()
    });
    reject("extra record", &|b| {
        b.local_decls
            .push(b.local_decls[matched.payload.record].clone());
    });
    reject("extra Box", &|b| {
        b.local_decls.push(b.local_decls[matched.owners[0]].clone());
    });
    reject("nominal definition", &|b| {
        let Rvalue::Aggregate(kind, _) = value(b, matched.payload.aggregate) else {
            unreachable!()
        };
        let mir::AggregateKind::Adt(def, ..) = &mut **kind else {
            unreachable!()
        };
        *def = other_plan.constructor.payload().definition().did();
    });
    reject("variant", &|b| {
        let Rvalue::Aggregate(kind, _) = value(b, matched.payload.aggregate) else {
            unreachable!()
        };
        let mir::AggregateKind::Adt(_, variant, ..) = &mut **kind else {
            unreachable!()
        };
        *variant = VariantIdx::from_usize(1);
    });
    reject("arguments", &|b| {
        let Rvalue::Aggregate(kind, _) = value(b, matched.payload.aggregate) else {
            unreachable!()
        };
        let mir::AggregateKind::Adt(_, _, args, ..) = &mut **kind else {
            unreachable!()
        };
        *args = tcx.mk_args(&[tcx.types.i32.into()]);
    });
    reject("union field", &|b| {
        let Rvalue::Aggregate(kind, _) = value(b, matched.payload.aggregate) else {
            unreachable!()
        };
        let mir::AggregateKind::Adt(_, _, _, _, active) = &mut **kind else {
            unreachable!()
        };
        *active = Some(first);
    });
    reject("duplicate operand", &|b| {
        let Rvalue::Aggregate(_, operands) = value(b, matched.payload.aggregate) else {
            unreachable!()
        };
        operands[second] = operands[first].clone();
    });
    reject("missing operand", &|b| {
        let Rvalue::Aggregate(_, operands) = value(b, matched.payload.aggregate) else {
            unreachable!()
        };
        operands.pop();
    });
    reject("extra operand", &|b| {
        let Rvalue::Aggregate(_, operands) = value(b, matched.payload.aggregate) else {
            unreachable!()
        };
        operands.push(operands[first].clone());
    });
    reject("copied aggregate operand", &|b| {
        let Rvalue::Aggregate(_, operands) = value(b, matched.payload.aggregate) else {
            unreachable!()
        };
        let Operand::Move(p) = operands[first] else {
            unreachable!()
        };
        operands[first] = Operand::Copy(p);
    });
    reject("wrong scalar producer", &|b| {
        let Rvalue::Use(operand, _) = value(b, matched.payload.fields[0].1) else {
            unreachable!()
        };
        *operand = Operand::Copy(mir::Place::from(mir::Local::from_usize(1)));
    });
    reject("moved scalar producer", &|b| {
        let Rvalue::Use(operand, _) = value(b, matched.payload.fields[0].1) else {
            unreachable!()
        };
        let Operand::Copy(p) = *operand else {
            unreachable!()
        };
        *operand = Operand::Move(p);
    });
    reject("initializer order", &|b| {
        let a = matched.payload.fields[0].1;
        let c = matched.payload.fields[1].1;
        b.basic_blocks.as_mut()[a.block]
            .statements
            .swap(a.statement_index, c.statement_index);
    });
    reject("copy non-Copy payload", &|b| {
        let Rvalue::Use(operand, _) = value(b, matched.payload.movement) else {
            unreachable!()
        };
        *operand = Operand::Copy(mir::Place::from(matched.payload.record));
    });
    reject("wrong call argument", &|b| {
        let TerminatorKind::Call { args, .. } = &mut b.basic_blocks.as_mut()[trace.call.0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        args[0].node = Operand::Move(mir::Place::from(matched.payload.record));
    });
    reject("copied call argument", &|b| {
        let TerminatorKind::Call { args, .. } = &mut b.basic_blocks.as_mut()[trace.call.0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        args[0].node = Operand::Copy(mir::Place::from(matched.payload.argument));
    });
    reject("copied Box", &|b| {
        let Rvalue::Use(operand, _) = value(b, matched.moves[0]) else {
            unreachable!()
        };
        *operand = Operand::Copy(mir::Place::from(matched.owners[0]));
    });
    reject("wrong Box move", &|b| {
        let Rvalue::Use(operand, _) = value(b, matched.moves[1]) else {
            unreachable!()
        };
        *operand = Operand::Move(mir::Place::from(matched.owners[0]));
    });
    reject("wrong selected field", &|b| {
        let Rvalue::Use(operand, _) = value(b, matched.read) else {
            unreachable!()
        };
        *operand = Operand::Copy(mir::Place::from(matched.pointer).project_deeper(
            &[
                mir::ProjectionElem::Deref,
                mir::ProjectionElem::Field(second, tcx.types.bool),
            ],
            tcx,
        ));
    });
    reject("wrong projected type", &|b| {
        let Rvalue::Use(operand, _) = value(b, matched.read) else {
            unreachable!()
        };
        *operand = Operand::Copy(mir::Place::from(matched.pointer).project_deeper(
            &[
                mir::ProjectionElem::Deref,
                mir::ProjectionElem::Field(first, tcx.types.u32),
            ],
            tcx,
        ));
    });
    let cast = trace.definition(matched.pointer).unwrap().location;
    reject("wrong pointer owner", &|b| {
        let Rvalue::Cast(_, Operand::Copy(p), _) = value(b, cast) else {
            unreachable!()
        };
        p.local = matched.owners[0];
    });
    reject("wrong drop owner", &|b| {
        let TerminatorKind::Drop { place, .. } = &mut b.basic_blocks.as_mut()[matched.drop.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *place = mir::Place::from(matched.owners[0]);
    });
    reject("missing drop", &|b| {
        let term = b.basic_blocks.as_mut()[matched.drop.block].terminator_mut();
        let TerminatorKind::Drop { target, .. } = term.kind else {
            unreachable!()
        };
        term.kind = TerminatorKind::Goto { target };
    });
    reject("unwind", &|b| {
        let TerminatorKind::Drop { unwind, .. } = &mut b.basic_blocks.as_mut()[matched.drop.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *unwind = mir::UnwindAction::Continue;
    });
    for location in [
        matched.payload.aggregate,
        matched.payload.movement,
        matched.payload.fields[0].1,
        matched.moves[0],
        cast,
        matched.read,
    ] {
        reject("missing assignment", &|b| {
            statement(b, location).kind = StatementKind::Nop
        });
    }
    reject("duplicate assignment", &|b| {
        let saved = statement(b, matched.moves[0]).clone();
        b.basic_blocks.as_mut()[matched.moves[0].block]
            .statements
            .push(saved);
    });
    let wrapper = tcx
        .hir_body_owners()
        .find(|id| tcx.def_path_str(*id) == "rejected::wrapper")
        .unwrap();
    let wrapper_body = tcx.mir_drops_elaborated_and_const_checked(wrapper).borrow();
    let wrapper_flow = flow::read(&wrapper_body).unwrap();
    let TerminatorKind::Call {
        func: wrapper_func, ..
    } = wrapper_flow.call.1
    else {
        unreachable!()
    };
    let other_body = tcx.mir_drops_elaborated_and_const_checked(other).borrow();
    let other_flow = flow::read(&other_body).unwrap();
    let TerminatorKind::Call {
        func: other_func, ..
    } = other_flow.call.1
    else {
        unreachable!()
    };
    reject("local wrapper callee", &|b| {
        let TerminatorKind::Call { func, .. } = &mut b.basic_blocks.as_mut()[trace.call.0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *func = wrapper_func.clone();
    });
    reject("constructor generic arguments", &|b| {
        let TerminatorKind::Call { func, .. } = &mut b.basic_blocks.as_mut()[trace.call.0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *func = other_func.clone();
    });
    reject("wrong same-type destination", &|b| {
        let TerminatorKind::Call { destination, .. } = &mut b.basic_blocks.as_mut()
            [trace.call.0.block]
            .terminator_mut()
            .kind
        else {
            unreachable!()
        };
        *destination = mir::Place::from(*matched.owners.last().unwrap());
    });
    assert_eq!(cases, 40);
    println!("40 boxed-record MIR corruptions rejected");
    let mut changed = source::read(tcx, owner).unwrap();
    changed.fields[0].parameter = changed.parameters[0].0;
    assert!(relations::validate(tcx, &changed, &original).is_err());
    let mut changed = source::read(tcx, owner).unwrap();
    changed.selected = second;
    assert!(relations::validate(tcx, &changed, &original).is_err());
    let mut changed = source::read(tcx, owner).unwrap();
    changed.scopes = other_plan.scopes;
    assert!(relations::validate(tcx, &changed, &original).is_err());
    println!("3 boxed-record source substitutions rejected");
    for (name, copied) in [("same_type", false), ("copied", true)] {
        let owner = tcx
            .hir_body_owners()
            .find(|id| tcx.def_path_str(*id) == name)
            .unwrap();
        let plan = source::read(tcx, owner).unwrap();
        let original = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let matched = relations::validate(tcx, &plan, &original).unwrap();
        let mut changed = original.clone();
        let (location, replacement) = if copied {
            (
                matched.payload.movement,
                Operand::Move(mir::Place::from(matched.payload.record)),
            )
        } else {
            assert_eq!(
                original.local_decls[mir::Local::from_usize(1)].ty,
                original.local_decls[mir::Local::from_usize(2)].ty
            );
            (
                matched.payload.fields[0].1,
                Operand::Copy(mir::Place::from(mir::Local::from_usize(1))),
            )
        };
        let Rvalue::Use(operand, _) = value(&mut changed, location) else {
            unreachable!()
        };
        *operand = replacement;
        assert!(
            relations::validate(tcx, &plan, &changed).is_err(),
            "admitted {name} corruption"
        );
    }
    println!("same-type parameter and Copy payload substitutions rejected");
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
