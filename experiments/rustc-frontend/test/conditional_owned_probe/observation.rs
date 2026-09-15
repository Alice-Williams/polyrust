//! Both normal outcomes, exact ownership events and canonical source exits.
use super::{
    events,
    source::{self, Case, Exit},
    trace::Trace,
};
use rustc_abi::FieldIdx;
use rustc_hir::{self as hir, def_id::LocalDefId};
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::{self, Ty, TyCtxt},
};
use std::collections::HashMap;

pub(super) fn check<'tcx>(tcx: TyCtxt<'tcx>, owner: LocalDefId, name: &str) -> Ty<'tcx> {
    let source = source::read(tcx, owner, Case::named(name));
    let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    assert_eq!(body.source.def_id(), owner.to_def_id());
    assert_eq!(
        body.phase,
        mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup)
    );
    assert_eq!(body.arg_count, 4);
    assert_eq!(body.local_decls[mir::RETURN_PLACE].ty, tcx.types.i32);
    let parameters: [mir::Local; 4] = body.args_iter().collect::<Vec<_>>().try_into().unwrap();
    for (source_parameter, parameter) in source.parameters.iter().zip(parameters) {
        assert_eq!(source_parameter.owner.def_id, owner);
        assert_eq!(
            body.local_decls[parameter].ty,
            tcx.typeck(owner).node_type(*source_parameter),
            "canonical HIR/MIR formal parameter correspondence"
        );
    }
    let paths = [
        Trace::read(tcx, &body, parameters[0], false),
        Trace::read(tcx, &body, parameters[0], true),
    ];
    assert_eq!(
        paths[0].visited.union(&paths[1].visited).count(),
        body.basic_blocks.len()
    );
    assert_eq!(paths[0].guard, paths[1].guard);
    if matches!(source.case, Case::Early(_)) {
        assert_eq!(paths[0].returning, paths[1].returning, "shared MIR Return");
    }
    let mut nominal = None;
    for (index, trace) in paths.iter().enumerate() {
        let choice = index == 1;
        let owners = events::constructors(tcx, owner, &source, &body, trace, &parameters);
        let record_ty =
            events::layout(tcx, owner, &source, owners[0].ty(&body.local_decls, tcx).ty);
        assert_eq!(record_ty, *nominal.get_or_insert(record_ty));
        let mut bindings: HashMap<_, _> = source
            .constructors
            .iter()
            .zip(&owners)
            .map(|((id, _), place)| (*id, *place))
            .collect();
        let mut ready_at = None;
        let record = if matches!(source.case, Case::Initialize { .. }) && !choice {
            assert!(
                !trace
                    .assignments
                    .iter()
                    .any(|(_, _, value)| matches!(value, Rvalue::Aggregate(..)))
            );
            None
        } else {
            let (at, aggregate) = events::aggregate(tcx, &source, &body, trace, record_ty, &owners);
            if matches!(source.case, Case::Initialize { .. }) {
                trace.before(trace.guard.unwrap().0, at);
                let (movement, record) = trace.moved(aggregate);
                trace.before(at, movement);
                ready_at = Some(movement);
                assert_eq!(record.ty(&body.local_decls, tcx).ty, record_ty);
                Some(record)
            } else {
                trace.before(at, trace.guard.unwrap().0);
                Some(aggregate)
            }
        };
        let taken = source.moves[index].map(|(binding, field)| {
            let (movement, taken) = trace.moved(events::field(tcx, &body, record.unwrap(), field));
            trace.before(trace.guard.unwrap().0, movement);
            ready_at = Some(movement);
            assert_eq!(
                taken.ty(&body.local_decls, tcx).ty,
                owners[0].ty(&body.local_decls, tcx).ty
            );
            assert!(taken.projection.is_empty());
            assert!(bindings.insert(binding, taken).is_none());
            taken
        });
        let mut expected = Vec::new();
        let flags = match source.case {
            Case::Initialize { .. } => {
                if choice {
                    expected.extend([record.unwrap(), owners[2]]);
                } else {
                    expected.extend([owners[2], owners[1], owners[0]]);
                }
                vec![choice, !choice, !choice]
            }
            Case::Partial(field) => {
                if choice {
                    expected.extend([
                        taken.unwrap(),
                        events::field(
                            tcx,
                            &body,
                            record.unwrap(),
                            FieldIdx::from_usize(1 - field.as_usize()),
                        ),
                        owners[2],
                    ]);
                } else {
                    expected.extend([
                        events::field(tcx, &body, record.unwrap(), FieldIdx::from_usize(0)),
                        events::field(tcx, &body, record.unwrap(), FieldIdx::from_usize(1)),
                        owners[2],
                    ]);
                }
                vec![!choice]
            }
            Case::Early(_) => {
                let (_, field) = source.moves[index].unwrap();
                expected.extend([
                    taken.unwrap(),
                    events::field(
                        tcx,
                        &body,
                        record.unwrap(),
                        FieldIdx::from_usize(1 - field.as_usize()),
                    ),
                    owners[2],
                ]);
                vec![]
            }
        };
        assert_eq!(
            trace
                .drops
                .iter()
                .map(|(_, place)| *place)
                .collect::<Vec<_>>(),
            expected,
            "exact normal cleanup"
        );
        assert_eq!(
            trace.decisions.iter().map(|d| d.value).collect::<Vec<_>>(),
            flags,
            "cleanup flag outcomes"
        );
        for decision in &trace.decisions {
            let (_, destination, value) = trace
                .assignments
                .iter()
                .find(|(at, _, _)| *at == decision.definition)
                .unwrap();
            assert_eq!(*destination, mir::Place::from(decision.local));
            let Rvalue::Use(Operand::Constant(value), _) = value else {
                panic!("flag definition")
            };
            assert_eq!(
                value
                    .const_
                    .try_eval_bool(tcx, ty::TypingEnv::fully_monomorphized()),
                Some(decision.value)
            );
            trace.before(decision.definition, decision.location);
        }
        let (cast, read) = scalar_read(tcx, &body, trace, bindings[&source.reads[index]]);
        trace.before(trace.guard.unwrap().0, cast);
        if let Some(movement) = ready_at {
            trace.before(movement, cast);
        }
        for (drop_index, &(at, _)) in trace.drops.iter().enumerate() {
            if matches!(source.case, Case::Partial(_)) && choice && drop_index == 0 {
                trace.before(ready_at.expect("branch field movement"), at);
                trace.before(at, cast);
            } else {
                trace.before(read, at);
            }
            trace.before(at, trace.returning);
        }
        let exit = &source.exits[index];
        let hir::Node::Expr(canonical) = tcx.hir_node(exit.value().hir_id) else {
            panic!("canonical source exit")
        };
        assert!(std::ptr::eq(canonical, exit.value()));
        match exit {
            Exit::Tail(value) => assert!(std::ptr::eq(source.root.expr.unwrap(), *value)),
            Exit::Return { expression, value } => {
                let hir::StmtKind::Semi(actual) = source.branch.stmts.last().unwrap().kind else {
                    panic!("early return statement")
                };
                assert!(std::ptr::eq(actual, *expression));
                assert!(
                    matches!(expression.kind, hir::ExprKind::Ret(Some(actual)) if std::ptr::eq(actual, *value))
                );
            }
        }
        assert_eq!(source.guard.hir_id.owner.def_id, owner);
        assert_eq!(
            trace.returning.statement_index,
            body.basic_blocks[trace.returning.block].statements.len()
        );
        println!("{name}/{choice}: exact source, typed events, flags, cleanup and exit observed");
    }
    nominal.unwrap()
}
fn scalar_read<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    owner: mir::Place<'tcx>,
) -> (mir::Location, mir::Location) {
    let (read, value) = trace.definition(mir::Place::from(mir::RETURN_PLACE));
    let Rvalue::Use(Operand::Copy(pointer), _) = value else {
        panic!("read")
    };
    assert_eq!(pointer.projection.as_slice(), &[mir::ProjectionElem::Deref]);
    let (cast, value) = trace.definition(mir::Place::from(pointer.local));
    let Rvalue::Cast(mir::CastKind::Transmute, Operand::Copy(source), target) = value else {
        panic!("pointer producer")
    };
    assert_eq!(source.local, owner.local);
    assert_eq!(*target, body.local_decls[pointer.local].ty);
    assert!(
        matches!(target.kind(), ty::RawPtr(payload, hir::Mutability::Not) if *payload == tcx.types.i32)
    );
    assert_eq!(source.projection.len(), 2);
    let mut expected = owner;
    for projection in source.projection {
        let mir::ProjectionElem::Field(index, actual) = projection else {
            panic!("pointer field")
        };
        assert_eq!(index.as_usize(), 0);
        expected = events::field(tcx, body, expected, index);
        assert_eq!(expected.ty(&body.local_decls, tcx).ty, actual);
    }
    assert_eq!(expected, *source);
    trace.before(cast, read);
    (cast, read)
}
