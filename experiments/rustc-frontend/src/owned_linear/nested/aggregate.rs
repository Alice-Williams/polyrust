//! One typed aggregate with source-ordered staging and declaration-indexed operands.
use super::super::{LinearError as E, Result, flow, source::local};
use super::{events, source::Plan};
use rustc_abi::FieldIdx;
use rustc_hir::{self as hir, HirId};
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::{Ty, TyCtxt},
};
use std::collections::{HashMap, HashSet};

pub(super) enum Which {
    Inner,
    Outer,
}
pub(super) struct Context<'a, 'mir, 'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub body: &'mir mir::Body<'tcx>,
    pub trace: &'a flow::Trace<'mir, 'tcx>,
    pub bindings: &'a mut HashMap<HirId, mir::Local>,
    pub storage: &'a mut HashSet<mir::Local>,
    pub used: &'a mut HashSet<mir::Location>,
}
pub(super) fn read<'tcx>(
    context: &mut Context<'_, '_, 'tcx>,
    plan: &Plan<'tcx>,
    which: Which,
    previous: mir::Location,
) -> Result<events::Aggregate<'tcx>> {
    let (binding, definition, arguments, result, initializers) = match which {
        Which::Inner => {
            let (binding, input) = &plan.inner;
            (
                *binding,
                input.definition(),
                input.arguments(),
                input.result(),
                input
                    .fields()
                    .iter()
                    .map(|f| (f.index(), f.ty(), f.initializer()))
                    .collect::<Vec<_>>(),
            )
        }
        Which::Outer => {
            let (binding, input) = &plan.outer;
            (
                *binding,
                input.layout().definition(),
                input.layout().arguments(),
                input.result(),
                input
                    .initializers()
                    .iter()
                    .map(|f| {
                        (
                            f.index(),
                            input.field(f.index()).unwrap().ty(),
                            f.expression(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        }
    };
    let mut matches = context.trace.assignments.iter().filter(|a| {
        matches!(a.value, Rvalue::Aggregate(..))
            && a.destination.ty(&context.body.local_decls, context.tcx).ty == result
    });
    let assignment = matches.next().ok_or(E::MoveGraph)?;
    if matches.next().is_some() || !assignment.destination.projection.is_empty() {
        return Err(E::MoveGraph);
    }
    let Rvalue::Aggregate(kind, operands) = assignment.value else {
        return Err(E::MoveGraph);
    };
    let mir::AggregateKind::Adt(actual, variant, args, annotation, active) = &**kind else {
        return Err(E::MoveGraph);
    };
    if *actual != definition.did()
        || variant.as_usize() != 0
        || *args != arguments
        || annotation.is_some()
        || active.is_some()
        || operands.len() != initializers.len()
        || !context.used.insert(assignment.location)
        || !context.storage.insert(assignment.destination.local)
    {
        return Err(E::MoveGraph);
    }
    let mut fields = Vec::new();
    let mut prior = previous;
    for (index, ty, initializer) in initializers {
        let Operand::Move(staging) = operands.get(index).ok_or(E::MoveGraph)? else {
            return Err(E::MoveGraph);
        };
        let field = match_field(
            context,
            plan,
            FieldInput {
                index,
                ty,
                initializer,
                staging: *staging,
                destination: assignment.destination,
                aggregate: assignment.location,
                previous: prior,
            },
        )?;
        prior = field.movement;
        fields.push(field);
    }
    if context
        .bindings
        .insert(binding, assignment.destination.local)
        .is_some()
    {
        return Err(E::SourceIdentity);
    }
    Ok(events::Aggregate {
        binding,
        destination: assignment.destination,
        location: assignment.location,
        fields,
    })
}

struct FieldInput<'tcx> {
    index: FieldIdx,
    ty: Ty<'tcx>,
    initializer: &'tcx hir::Expr<'tcx>,
    staging: mir::Place<'tcx>,
    destination: mir::Place<'tcx>,
    aggregate: mir::Location,
    previous: mir::Location,
}
fn match_field<'tcx>(
    context: &mut Context<'_, '_, 'tcx>,
    plan: &Plan<'tcx>,
    input: FieldInput<'tcx>,
) -> Result<events::Field<'tcx>> {
    let source_binding = local(context.tcx.typeck(plan.frame.owner), input.initializer)?;
    let source = mir::Place::from(
        *context
            .bindings
            .get(&source_binding)
            .ok_or(E::SourceIdentity)?,
    );
    if !input.staging.projection.is_empty()
        || input.staging.ty(&context.body.local_decls, context.tcx).ty != input.ty
        || !context.storage.insert(input.staging.local)
    {
        return Err(E::MoveGraph);
    }
    let movement = context.trace.definition(input.staging.local)?;
    if movement.destination != input.staging
        || !matches!(movement.value, Rvalue::Use(Operand::Move(actual), _) if *actual == source)
        || source.ty(&context.body.local_decls, context.tcx).ty != input.ty
        || !context.trace.before(input.previous, movement.location)
        || !context.trace.before(movement.location, input.aggregate)
        || !context.used.insert(movement.location)
    {
        return Err(E::MoveGraph);
    }
    let destination = input.destination.project_deeper(
        &[mir::ProjectionElem::Field(input.index, input.ty)],
        context.tcx,
    );
    Ok(events::Field {
        initializer: input.initializer.hir_id,
        index: input.index,
        source: (source_binding, source),
        staging: input.staging,
        movement: movement.location,
        destination,
    })
}
