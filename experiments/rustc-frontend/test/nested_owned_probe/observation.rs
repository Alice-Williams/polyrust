//! Exact source and typed MIR observations; intentionally no certificate API.
use super::trace::Trace;
use rustc_abi::FieldIdx;
use rustc_hir::{self as hir, def_id::LocalDefId};
use rustc_middle::{
    mir::{self, Operand, Rvalue, TerminatorKind},
    ty::{self, Ty, TyCtxt},
};

pub(super) fn check<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    name: &str,
) -> (Ty<'tcx>, Ty<'tcx>) {
    let source = tcx.hir_body_owned_by(owner);
    let hir::ExprKind::Block(block, None) = source.value.kind else {
        panic!("root block")
    };
    let multiple = name == "multiple";
    let whole = name == "whole_inner";
    let reverse = name == "reversed";
    assert_eq!(block.stmts.len(), if multiple || whole { 7 } else { 6 });
    assert_eq!(source.params.len(), 3);
    for parameter in source.params {
        assert!(
            matches!(
                parameter.pat.kind,
                hir::PatKind::Binding(rustc_ast::BindingMode::NONE, _, _, None)
            ),
            "immutable parameter"
        );
    }
    let declarations: Vec<_> = block
        .stmts
        .iter()
        .map(|statement| {
            let hir::StmtKind::Let(local) = statement.kind else {
                panic!("let")
            };
            assert!(local.els.is_none());
            assert!(
                matches!(
                    local.pat.kind,
                    hir::PatKind::Binding(rustc_ast::BindingMode::NONE, _, _, None)
                ),
                "immutable binding"
            );
            local
        })
        .collect();
    let typeck = tcx.typeck(owner);
    let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    assert_eq!(body.source.def_id(), owner.to_def_id());
    assert_eq!(
        body.phase,
        mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup)
    );
    assert_eq!(body.arg_count, 3);
    assert_eq!(body.local_decls[mir::RETURN_PLACE].ty, tcx.types.i32);
    let trace = Trace::read(&body);
    assert_eq!(trace.calls.len(), 3);
    assert_eq!(trace.drops.len(), 3);
    let mut owners = Vec::new();
    for (index, &(at, call)) in trace.calls.iter().enumerate() {
        let TerminatorKind::Call {
            func,
            args,
            destination,
            ..
        } = call
        else {
            unreachable!()
        };
        assert_eq!(args.len(), 1);
        let expression = declarations[index].init.unwrap();
        let hir::ExprKind::Call(callee, source_args) = expression.kind else {
            panic!("constructor")
        };
        assert_eq!(source_args.len(), 1);
        assert_eq!(func.ty(&body.local_decls, tcx), typeck.expr_ty(callee));
        let constructor = tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap();
        let constructor_args = tcx.mk_args(&[tcx.types.i32.into()]);
        assert!(matches!(func, Operand::Constant(_)));
        assert!(
            matches!(func.ty(&body.local_decls, tcx).kind(), ty::FnDef(definition, arguments) if *definition == constructor && *arguments == constructor_args)
        );
        let box_ty = typeck.expr_ty(expression);
        let ty::Adt(definition, arguments) = box_ty.kind() else {
            panic!("Box")
        };
        assert_eq!(Some(definition.did()), tcx.lang_items().owned_box());
        assert_eq!(arguments.type_at(0), tcx.types.i32);
        assert_eq!(destination.ty(&body.local_decls, tcx).ty, box_ty);
        let Operand::Move(stage) = args[0].node else {
            panic!("scalar argument stage")
        };
        let (copy_at, value) = trace.definition(stage);
        let expected = mir::Place::from(mir::Local::from_usize(index + 1));
        assert!(matches!(value, Rvalue::Use(Operand::Copy(actual), _) if *actual == expected));
        assert_eq!(expected.ty(&body.local_decls, tcx).ty, tcx.types.i32);
        trace.before(copy_at, at);
        owners.push(*destination);
    }
    let inner_ty = typeck.node_type(declarations[3].pat.hir_id);
    let outer_ty = typeck.node_type(declarations[4].pat.hir_id);
    assert_ne!(inner_ty, outer_ty);
    let inner = aggregate(tcx, &body, &trace, inner_ty, &owners[..2], reverse);
    let outer = aggregate(tcx, &body, &trace, outer_ty, &[inner.1, owners[2]], reverse);
    trace.before(trace.calls[2].0, inner.0);
    trace.before(inner.0, outer.0);
    let nested = field(tcx, &body, outer.1, 0);
    assert_eq!(nested.ty(&body.local_decls, tcx).ty, inner_ty);
    let selected_index = usize::from(name == "second");
    let (container, before) = if whole {
        let (at, moved) = trace.moved(nested);
        trace.before(outer.0, at);
        assert_eq!(moved.ty(&body.local_decls, tcx).ty, inner_ty);
        (moved, at)
    } else {
        (nested, outer.0)
    };
    let (take_at, taken) = trace.moved(field(tcx, &body, container, selected_index));
    trace.before(before, take_at);
    let mut expected_drops = vec![];
    if multiple {
        let (second_at, second) = trace.moved(field(tcx, &body, nested, 1));
        trace.before(take_at, second_at);
        expected_drops.push(second);
    }
    expected_drops.push(taken);
    if !multiple {
        expected_drops.push(field(tcx, &body, container, 1 - selected_index));
    }
    expected_drops.push(field(tcx, &body, outer.1, 1));
    assert_eq!(
        trace
            .drops
            .iter()
            .map(|(_, place)| *place)
            .collect::<Vec<_>>(),
        expected_drops
    );
    let (read_at, value) = trace.definition(mir::Place::from(mir::RETURN_PLACE));
    let Rvalue::Use(Operand::Copy(read), _) = value else {
        panic!("scalar read")
    };
    assert_eq!(read.projection.as_ref(), &[mir::ProjectionElem::Deref]);
    let (cast_at, cast) = trace.definition(mir::Place::from(read.local));
    let Rvalue::Cast(_, Operand::Copy(pointer), target) = cast else {
        panic!("pointer cast")
    };
    assert_eq!(pointer.local, taken.local);
    assert!(
        matches!(target.kind(), ty::RawPtr(payload, hir::Mutability::Not) if *payload == tcx.types.i32)
    );
    trace.before(take_at, cast_at);
    trace.before(cast_at, read_at);
    let mut previous = read_at;
    for &(at, place) in &trace.drops {
        assert_eq!(
            place.ty(&body.local_decls, tcx).ty,
            owners[0].ty(&body.local_decls, tcx).ty
        );
        trace.before(previous, at);
        previous = at;
    }
    trace.before(previous, trace.returning);
    println!("{name}: nested aggregate, transfer, read and cleanup observations passed");
    (inner_ty, outer_ty)
}

fn field<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    place: mir::Place<'tcx>,
    index: usize,
) -> mir::Place<'tcx> {
    let ty::Adt(definition, arguments) = place.ty(&body.local_decls, tcx).ty.kind() else {
        panic!("record path")
    };
    assert!(definition.is_struct() && !definition.has_dtor(tcx));
    let index = FieldIdx::from_usize(index);
    let ty = tcx
        .try_normalize_erasing_regions(
            ty::TypingEnv::fully_monomorphized(),
            definition.non_enum_variant().fields[index].ty(tcx, arguments),
        )
        .unwrap();
    place.project_deeper(&[mir::ProjectionElem::Field(index, ty)], tcx)
}
fn aggregate<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    ty: Ty<'tcx>,
    sources: &[mir::Place<'tcx>],
    reverse: bool,
) -> (mir::Location, mir::Place<'tcx>) {
    let ty::Adt(definition, arguments) = ty.kind() else {
        panic!("record type")
    };
    assert!(definition.is_struct() && !definition.has_dtor(tcx));
    assert!(definition.did().is_local());
    assert!(arguments.is_empty());
    assert_eq!(tcx.generics_of(definition.did()).count(), 0);
    assert!(
        definition.non_enum_variant().ctor_kind().is_none(),
        "named-field record"
    );
    assert_eq!(definition.non_enum_variant().fields.len(), sources.len());
    let matching: Vec<_> = trace.assignments.iter().filter(|(_, _, value)| matches!(value, Rvalue::Aggregate(kind, _) if matches!(**kind, mir::AggregateKind::Adt(actual, ..) if actual == definition.did()))).collect();
    assert_eq!(matching.len(), 1);
    let (at, destination, value) = *matching[0];
    assert_eq!(destination.ty(&body.local_decls, tcx).ty, ty);
    let Rvalue::Aggregate(kind, operands) = value else {
        unreachable!()
    };
    let mir::AggregateKind::Adt(_, variant, actual_args, _, union_field) = **kind else {
        unreachable!()
    };
    assert_eq!(variant.as_usize(), 0);
    assert_eq!(actual_args, *arguments);
    assert!(union_field.is_none());
    assert_eq!(operands.len(), sources.len());
    let mut stages = Vec::new();
    for (index, operand) in operands.iter_enumerated() {
        let Operand::Move(stage) = *operand else {
            panic!("aggregate move")
        };
        let (stage_at, staged) = trace.definition(stage);
        assert!(
            matches!(staged, Rvalue::Use(Operand::Move(source), _) if *source == sources[index.as_usize()])
        );
        assert_eq!(
            stage.ty(&body.local_decls, tcx).ty,
            tcx.try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                definition.non_enum_variant().fields[index].ty(tcx, arguments)
            )
            .unwrap()
        );
        trace.before(stage_at, at);
        stages.push(stage_at);
    }
    if reverse {
        trace.before(stages[1], stages[0]);
    } else {
        trace.before(stages[0], stages[1]);
    }
    (at, destination)
}
