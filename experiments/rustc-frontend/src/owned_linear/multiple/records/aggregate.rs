//! Declaration-indexed aggregate operands and source-ordered staging moves.
use super::{
    super::super::{LinearError as Error, Result, flow},
    source::Plan,
};
use rustc_abi::FieldIdx;
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::TyCtxt,
};
use std::collections::HashSet;

pub(super) struct Field<'tcx> {
    pub index: FieldIdx,
    pub source: mir::Place<'tcx>,
    pub staging: mir::Place<'tcx>,
    pub place: mir::Place<'tcx>,
    pub movement: mir::Location,
}
pub(super) struct Aggregate<'tcx> {
    pub location: mir::Location,
    pub destination: mir::Place<'tcx>,
    /// Source initializer order, not declaration order.
    pub fields: Vec<Field<'tcx>>,
}

pub(super) fn read<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &flow::Trace<'_, 'tcx>,
) -> Result<Aggregate<'tcx>> {
    let mut aggregates = trace
        .assignments
        .iter()
        .filter(|a| matches!(a.value, Rvalue::Aggregate(..)));
    let assignment = aggregates.next().ok_or(Error::MoveGraph)?;
    if aggregates.next().is_some()
        || assignment.destination.ty(&body.local_decls, tcx).ty != plan.record.result()
    {
        return Err(Error::MoveGraph);
    }
    let Rvalue::Aggregate(kind, operands) = assignment.value else {
        unreachable!()
    };
    let mir::AggregateKind::Adt(definition, variant, arguments, annotation, active) = &**kind
    else {
        return Err(Error::MoveGraph);
    };
    if *definition != plan.record.definition().did()
        || variant.as_usize() != 0
        || *arguments != plan.record.arguments()
        || annotation.is_some()
        || active.is_some()
        || operands.len() != plan.record.fields().len()
    {
        return Err(Error::MoveGraph);
    }
    let mut fields = Vec::new();
    let mut seen = HashSet::new();
    let mut previous = None;
    for field in plan.record.fields() {
        let Operand::Move(staging) = operands.get(field.index()).ok_or(Error::MoveGraph)? else {
            return Err(Error::MoveGraph);
        };
        if !staging.projection.is_empty()
            || staging.ty(&body.local_decls, tcx).ty != field.ty()
            || !seen.insert(staging.local)
        {
            return Err(Error::MoveGraph);
        }
        let movement = trace.definition(staging.local)?;
        let Rvalue::Use(Operand::Move(source), _) = movement.value else {
            return Err(Error::MoveGraph);
        };
        if !source.projection.is_empty()
            || source.ty(&body.local_decls, tcx).ty != field.ty()
            || !trace.before(movement.location, assignment.location)
            || previous.is_some_and(|p| !trace.before(p, movement.location))
        {
            return Err(Error::MoveGraph);
        }
        previous = Some(movement.location);
        fields.push(Field {
            index: field.index(),
            source: *source,
            staging: *staging,
            place: assignment.destination.project_deeper(
                &[mir::ProjectionElem::Field(field.index(), field.ty())],
                tcx,
            ),
            movement: movement.location,
        });
    }
    Ok(Aggregate {
        location: assignment.location,
        destination: assignment.destination,
        fields,
    })
}
