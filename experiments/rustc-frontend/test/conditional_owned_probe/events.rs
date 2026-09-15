//! Typed constructors and source-ordered aggregate staging in each observed path.
use super::{
    source::{self, Case, Source},
    trace::Trace,
};
use rustc_abi::FieldIdx;
use rustc_hir::{self as hir, def_id::LocalDefId};
use rustc_middle::{
    mir::{self, Operand, Rvalue, TerminatorKind},
    ty::{self, Ty, TyCtxt},
};

pub(super) fn constructors<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    source: &Source<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    parameters: &[mir::Local; 4],
) -> Vec<mir::Place<'tcx>> {
    assert_eq!(trace.calls.len(), 3);
    let checked = tcx.typeck(owner);
    let constructor = tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap();
    let arguments = tcx.mk_args(&[tcx.types.i32.into()]);
    let box_ty = tcx
        .fn_sig(constructor)
        .instantiate(tcx, arguments)
        .skip_binder()
        .output();
    let mut owners = Vec::new();
    let mut previous = None;
    for (index, &(at, call)) in trace.calls.iter().enumerate() {
        let TerminatorKind::Call {
            func: func @ Operand::Constant(_),
            args,
            destination,
            ..
        } = call
        else {
            panic!("constructor call")
        };
        assert_eq!(
            func.ty(&body.local_decls, tcx),
            Ty::new_fn_def(tcx, constructor, arguments),
            "standard Box constructor"
        );
        let expression = source.constructors[index].1;
        let hir::ExprKind::Call(callee, _) = expression.kind else {
            panic!("source constructor")
        };
        assert_eq!(checked.expr_ty(callee), func.ty(&body.local_decls, tcx));
        assert!(checked.expr_adjustments(expression).is_empty());
        assert_eq!(checked.expr_ty(expression), box_ty);
        assert_eq!(destination.ty(&body.local_decls, tcx).ty, box_ty);
        assert!(destination.projection.is_empty());
        assert_eq!(args.len(), 1);
        let Operand::Move(stage) = args[0].node else {
            panic!("scalar staging")
        };
        let (copy_at, value) = trace.definition(stage);
        let parameter = mir::Place::from(parameters[index + 1]);
        assert!(
            matches!(value, Rvalue::Use(Operand::Copy(actual), _) if *actual == parameter),
            "scalar MIR parameter anchor"
        );
        assert_eq!(parameter.ty(&body.local_decls, tcx).ty, tcx.types.i32);
        assert_eq!(stage.ty(&body.local_decls, tcx).ty, tcx.types.i32);
        assert!(stage.projection.is_empty());
        if let Some(previous) = previous {
            trace.before(previous, copy_at);
        }
        trace.before(copy_at, at);
        previous = Some(at);
        owners.push(*destination);
    }
    trace.before(previous.unwrap(), trace.guard.unwrap().1);
    assert_eq!(
        owners
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        3,
        "three distinct simultaneously live constructor owners"
    );
    owners
}
pub(super) fn layout<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    source: &Source<'tcx>,
    box_ty: Ty<'tcx>,
) -> Ty<'tcx> {
    let checked = tcx.typeck(owner);
    let ty = checked.expr_ty(source.record);
    assert_eq!(ty, checked.node_type(source.record_binding));
    let ty::Adt(definition, arguments) = ty.kind() else {
        panic!("Pair type")
    };
    assert!(definition.is_struct() && definition.did().is_local() && !definition.has_dtor(tcx));
    assert!(arguments.is_empty());
    assert_eq!(tcx.generics_of(definition.did()).count(), 0);
    assert!(
        definition.non_enum_variant().ctor_kind().is_none(),
        "named-field Pair"
    );
    assert!(
        definition.repr().flags.is_empty()
            && definition.repr().pack.is_none()
            && definition.repr().align.is_none(),
        "default record representation"
    );
    let hir::ExprKind::Struct(_, fields, hir::StructTailExpr::None) = source.record.kind else {
        panic!("complete record initializer")
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(definition.non_enum_variant().fields.len(), 2);
    let reversed = matches!(source.case, Case::Initialize { reversed: true });
    for (position, field) in fields.iter().enumerate() {
        let index = checked.field_index(field.hir_id);
        assert_eq!(
            index.as_usize(),
            if reversed { 1 - position } else { position },
            "initializer source order"
        );
        assert_eq!(
            source::local(checked, field.expr),
            source.constructors[index.as_usize()].0,
            "field initializer owner"
        );
        let declared = &definition.non_enum_variant().fields[index];
        let actual = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                declared.ty(tcx, arguments),
            )
            .unwrap();
        assert_eq!(actual, box_ty);
    }
    ty
}
pub(super) fn aggregate<'tcx>(
    tcx: TyCtxt<'tcx>,
    source: &Source<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    record_ty: Ty<'tcx>,
    owners: &[mir::Place<'tcx>],
) -> (mir::Location, mir::Place<'tcx>) {
    let found: Vec<_> = trace
        .assignments
        .iter()
        .filter(|(_, _, value)| matches!(value, Rvalue::Aggregate(..)))
        .collect();
    assert_eq!(found.len(), 1);
    let (at, destination, value) = *found[0];
    assert_eq!(destination.ty(&body.local_decls, tcx).ty, record_ty);
    let ty::Adt(definition, arguments) = record_ty.kind() else {
        panic!("record type")
    };
    let Rvalue::Aggregate(kind, operands) = value else {
        panic!("aggregate")
    };
    let mir::AggregateKind::Adt(actual, variant, args, annotation, union) = &**kind else {
        panic!("nominal aggregate")
    };
    assert_eq!(*actual, definition.did());
    assert_eq!(*args, *arguments);
    assert_eq!(variant.as_usize(), 0);
    assert!(annotation.is_none() && union.is_none());
    assert_eq!(operands.len(), 2);
    let mut stages = Vec::new();
    for (index, operand) in operands.iter_enumerated() {
        let Operand::Move(stage) = *operand else {
            panic!("aggregate staging operand")
        };
        assert!(stage.projection.is_empty());
        assert_eq!(
            stage.ty(&body.local_decls, tcx).ty,
            owners[index.as_usize()].ty(&body.local_decls, tcx).ty
        );
        let (movement, value) = trace.definition(stage);
        assert!(
            matches!(value, Rvalue::Use(Operand::Move(actual), _) if *actual == owners[index.as_usize()])
        );
        trace.before(trace.calls[2].0, movement);
        if matches!(source.case, Case::Initialize { .. }) {
            trace.before(trace.guard.unwrap().0, movement);
        }
        trace.before(movement, at);
        stages.push(movement);
    }
    if matches!(source.case, Case::Initialize { reversed: true }) {
        trace.before(stages[1], stages[0]);
    } else {
        trace.before(stages[0], stages[1]);
    }
    (at, destination)
}
pub(super) fn field<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    record: mir::Place<'tcx>,
    index: FieldIdx,
) -> mir::Place<'tcx> {
    let ty::Adt(definition, arguments) = record.ty(&body.local_decls, tcx).ty.kind() else {
        panic!("record field")
    };
    let ty = tcx
        .try_normalize_erasing_regions(
            ty::TypingEnv::fully_monomorphized(),
            definition.non_enum_variant().fields[index].ty(tcx, arguments),
        )
        .unwrap();
    record.project_deeper(&[mir::ProjectionElem::Field(index, ty)], tcx)
}
