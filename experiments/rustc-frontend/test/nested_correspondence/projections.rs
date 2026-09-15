//! Assert concrete compiler events independently of private relation state.
use super::trace::Trace;
use crate::owned_linear::nested::{NestedOwnedBody, path::SourcePlace};
use rustc_middle::{
    mir::{self, Operand, Rvalue, TerminatorKind},
    ty::{self, TyCtxt},
};

pub(super) fn events<'tcx>(
    tcx: TyCtxt<'tcx>,
    proof: &NestedOwnedBody<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    reversed: bool,
) {
    let parameters: Vec<_> = body.args_iter().collect();
    let mut previous = None;
    for (index, (construction, input)) in proof
        .constructions()
        .iter()
        .zip(proof.box_inputs())
        .enumerate()
    {
        let rustc_hir::ExprKind::Path(ref path) = input.argument().kind else {
            panic!("parameter path")
        };
        let rustc_hir::def::Res::Local(parameter_id) = tcx
            .typeck(proof.owner())
            .qpath_res(path, input.argument().hir_id)
        else {
            panic!("parameter binding")
        };
        let position = tcx.hir_body_owned_by(proof.owner()).params.iter().position(|parameter| matches!(parameter.pat.kind, rustc_hir::PatKind::Binding(_, id, _, None) if id == parameter_id)).unwrap();
        assert_eq!(
            construction.parameter(),
            (parameter_id, parameters[position])
        );
        assert_eq!(construction.binding(), proof.bindings()[index]);
        assert_eq!(construction.parameter().0.owner.def_id, proof.owner());
        assert_eq!(construction.call(), trace.calls[index].0);
        let TerminatorKind::Call {
            func,
            args,
            destination,
            ..
        } = trace.calls[index].1
        else {
            panic!("call");
        };
        assert!(matches!(func, Operand::Constant(_)));
        assert_eq!(
            func.ty(&body.local_decls, tcx),
            Ty::new_fn_def(tcx, input.constructor(), input.arguments())
        );
        assert_eq!(*destination, mir::Place::from(construction.binding().1));
        let [argument] = &args[..] else {
            panic!("argument");
        };
        let Operand::Move(stage) = argument.node else {
            panic!("scalar staging");
        };
        let (at, value) = trace.definition(stage);
        assert_eq!(construction.scalar_staging(), &[at]);
        assert!(
            matches!(value, Rvalue::Use(Operand::Copy(place), _) if *place == mir::Place::from(parameters[position]))
        );
        if let Some(previous) = previous {
            trace.before(previous, at);
        }
        trace.before(at, construction.call());
        previous = Some(construction.call());
    }
    let (inner, outer) = proof.record_inputs();
    for (number, aggregate) in proof.aggregates().iter().enumerate() {
        assert_eq!(
            (aggregate.binding(), aggregate.destination().local),
            proof.bindings()[number + 3]
        );
        let (at, value) = trace.definition(aggregate.destination());
        assert_eq!(at, aggregate.location());
        let Rvalue::Aggregate(kind, operands) = value else {
            panic!("aggregate");
        };
        let mir::AggregateKind::Adt(definition, variant, arguments, annotation, active) = &**kind
        else {
            panic!("ADT");
        };
        let expected = if number == 0 {
            (inner.definition(), inner.arguments())
        } else {
            (outer.layout().definition(), outer.layout().arguments())
        };
        assert_eq!(*definition, expected.0.did());
        assert_eq!(*arguments, expected.1);
        assert_eq!(variant.as_usize(), 0);
        assert!(annotation.is_none() && active.is_none());
        assert_eq!(operands.len(), 2);
        assert_eq!(
            aggregate
                .fields()
                .iter()
                .map(|f| f.index().as_usize())
                .collect::<Vec<_>>(),
            if reversed { vec![1, 0] } else { vec![0, 1] }
        );
        for field in aggregate.fields() {
            let expected_initializer = if number == 0 {
                inner
                    .fields()
                    .iter()
                    .find(|f| f.index() == field.index())
                    .unwrap()
                    .initializer()
            } else {
                outer
                    .initializers()
                    .iter()
                    .find(|f| f.index() == field.index())
                    .unwrap()
                    .expression()
            };
            assert_eq!(field.initializer(), expected_initializer.hir_id);
            assert_eq!(
                field.source().1,
                mir::Place::from(
                    proof
                        .bindings()
                        .iter()
                        .find(|(id, _)| *id == field.source().0)
                        .unwrap()
                        .1
                )
            );
            assert_eq!(operands[field.index()], Operand::Move(field.staging()));
            let (movement, staging) = trace.moved(field.source().1);
            assert_eq!((movement, staging), (field.movement(), field.staging()));
            trace.before(previous.unwrap(), movement);
            trace.before(movement, at);
            previous = Some(movement);
            let declared = &expected.0.non_enum_variant().fields[field.index()];
            let ty = tcx
                .try_normalize_erasing_regions(
                    ty::TypingEnv::fully_monomorphized(),
                    declared.ty(tcx, expected.1),
                )
                .unwrap();
            assert_eq!(
                field.destination(),
                aggregate
                    .destination()
                    .project_deeper(&[mir::ProjectionElem::Field(field.index(), ty)], tcx)
            );
        }
        previous = Some(at);
    }
    for movement in proof.movements() {
        let root = proof
            .bindings()
            .iter()
            .find(|(id, _)| *id == movement.source().binding())
            .unwrap()
            .1;
        path(tcx, body, movement.source(), root, movement.actual_source());
        let (at, destination) = trace.moved(movement.actual_source());
        assert_eq!(
            (at, destination),
            (
                movement.location(),
                mir::Place::from(movement.destination().1)
            )
        );
        assert!(proof.bindings().contains(&movement.destination()));
        trace.before(previous.unwrap(), at);
        previous = Some(at);
    }
}
use rustc_middle::ty::Ty;
pub(super) fn pointer<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    source: mir::Place<'tcx>,
    target: Ty<'tcx>,
    pointer: mir::Local,
) {
    assert_eq!(target, body.local_decls[pointer].ty);
    assert!(
        matches!(target.kind(), ty::RawPtr(payload, rustc_hir::Mutability::Not) if *payload == tcx.types.i32)
    );
    assert_eq!(source.projection.len(), 2);
    let mut expected = mir::Place::from(source.local);
    for projection in source.projection {
        let mir::ProjectionElem::Field(index, actual) = projection else {
            panic!("pointer field")
        };
        assert_eq!(index.as_usize(), 0);
        let ty::Adt(definition, arguments) = expected.ty(&body.local_decls, tcx).ty.kind() else {
            panic!("pointer parent")
        };
        assert!(definition.is_struct());
        let declared = &definition.non_enum_variant().fields[index];
        let ty = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                declared.ty(tcx, arguments),
            )
            .unwrap();
        assert_eq!(actual, ty);
        expected = expected.project_deeper(&[mir::ProjectionElem::Field(index, ty)], tcx);
    }
    assert_eq!(expected, source);
}
pub(super) fn cleanup<'tcx>(
    tcx: TyCtxt<'tcx>,
    proof: &NestedOwnedBody<'tcx>,
    body: &mir::Body<'tcx>,
    name: &str,
) {
    use crate::owned_linear::nested::events::CleanupKind;
    let grouped = matches!(name, "spare" | "whole_unopened");
    assert_eq!(proof.cleanup().len(), if grouped { 2 } else { 3 });
    for (index, cleanup) in proof.cleanup().iter().enumerate() {
        let root = proof
            .bindings()
            .iter()
            .find(|(id, _)| *id == cleanup.source().binding())
            .unwrap()
            .1;
        path(tcx, body, cleanup.source(), root, cleanup.actual());
        let leaves: Vec<_> = proof
            .leaves()
            .iter()
            .filter(|leaf| leaf.cleanup_location() == cleanup.location())
            .collect();
        assert_eq!(
            cleanup.constructors(),
            leaves
                .iter()
                .map(|leaf| leaf.constructor())
                .collect::<Vec<_>>()
        );
        if grouped && index == 1 {
            assert_eq!(cleanup.kind(), CleanupKind::InnerRecord);
            assert_eq!(
                cleanup.actual().ty(&body.local_decls, tcx).ty,
                proof.record_inputs().0.result()
            );
            assert_eq!(
                cleanup.constructors(),
                &[proof.bindings()[0].0, proof.bindings()[1].0]
            );
            let (binding, fields): (_, &[usize]) =
                if name == "spare" { (4, &[0]) } else { (5, &[]) };
            assert_eq!(cleanup.source().binding(), proof.bindings()[binding].0);
            assert_eq!(
                cleanup
                    .source()
                    .fields()
                    .iter()
                    .map(|f| f.index().as_usize())
                    .collect::<Vec<_>>(),
                fields
            );
        } else {
            assert_eq!(cleanup.kind(), CleanupKind::Leaf);
            assert_eq!(leaves.len(), 1);
            assert_eq!(cleanup.source(), leaves[0].source());
            assert_eq!(cleanup.actual(), leaves[0].actual());
        }
    }
}
pub(super) fn path<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    path: &SourcePlace<'tcx>,
    root: mir::Local,
    actual: mir::Place<'tcx>,
) {
    let mut expected = mir::Place::from(root);
    for field in path.fields() {
        let parent = expected.ty(&body.local_decls, tcx).ty;
        assert_eq!(parent, field.parent());
        let ty::Adt(definition, arguments) = parent.kind() else {
            panic!("parent");
        };
        let declared = &definition.non_enum_variant().fields[field.index()];
        assert_eq!(declared.did, field.declaration());
        let ty = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                declared.ty(tcx, arguments),
            )
            .unwrap();
        assert_eq!(ty, field.ty());
        expected = expected.project_deeper(&[mir::ProjectionElem::Field(field.index(), ty)], tcx);
    }
    assert_eq!(actual, expected);
}
