//! Parameter-copy staging and the distinct record-to-argument transfer.
use super::super::super::{LinearError as Error, Result, flow::Trace};
use super::{PayloadTransfer, source::Plan};
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::{self, TyCtxt},
};
use std::collections::HashSet;

pub(super) struct Matched {
    pub record: mir::Local,
    pub argument: mir::Local,
    pub transfer: PayloadTransfer,
    pub aggregate: mir::Location,
    pub movement: mir::Location,
    /// Scalar staging locals in source initializer order.
    pub fields: Vec<(mir::Local, mir::Location)>,
}
pub(super) fn read<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    argument: &Operand<'tcx>,
    call: mir::Location,
    used: &mut HashSet<mir::Location>,
) -> Result<Matched> {
    let payload = plan.constructor.payload();
    let Operand::Move(argument) = argument else {
        return Err(Error::Argument);
    };
    if !argument.projection.is_empty() || argument.ty(&body.local_decls, tcx).ty != payload.ty() {
        return Err(Error::Argument);
    }
    let movement = trace.definition(argument.local)?;
    let Rvalue::Use(operand, _) = movement.value else {
        return Err(Error::Argument);
    };
    let (transfer, record) = match operand {
        Operand::Move(place) => (PayloadTransfer::Move, place),
        Operand::Copy(place) => (PayloadTransfer::Copy, place),
        _ => return Err(Error::Argument),
    };
    let copies =
        tcx.type_is_copy_modulo_regions(ty::TypingEnv::fully_monomorphized(), payload.ty());
    if (transfer == PayloadTransfer::Copy) != copies
        || !record.projection.is_empty()
        || record.ty(&body.local_decls, tcx).ty != payload.ty()
        || record.local == argument.local
        || !trace.before(movement.location, call)
        || !used.insert(movement.location)
    {
        return Err(Error::Argument);
    }
    let aggregate = trace.definition(record.local)?;
    let Rvalue::Aggregate(kind, operands) = aggregate.value else {
        return Err(Error::Argument);
    };
    let mir::AggregateKind::Adt(definition, variant, arguments, annotation, active) = &**kind
    else {
        return Err(Error::Argument);
    };
    if *definition != payload.definition().did()
        || variant.as_usize() != 0
        || *arguments != payload.arguments()
        || annotation.is_some()
        || active.is_some()
        || operands.len() != plan.fields.len()
        || !trace.before(aggregate.location, movement.location)
        || !used.insert(aggregate.location)
    {
        return Err(Error::Argument);
    }
    let parameters: Vec<_> = body.args_iter().collect();
    let mut fields = Vec::new();
    let mut staging = HashSet::new();
    let mut previous = None;
    for field in &plan.fields {
        let expected = payload.fields()[field.index.as_usize()].ty();
        let position = plan
            .parameters
            .iter()
            .position(|p| p.0 == field.parameter)
            .ok_or(Error::Argument)?;
        let parameter = parameters[position];
        let Operand::Move(local) = operands.get(field.index).ok_or(Error::Argument)? else {
            return Err(Error::Argument);
        };
        if !local.projection.is_empty()
            || parameters.contains(&local.local)
            || local.ty(&body.local_decls, tcx).ty != expected
            || !staging.insert(local.local)
        {
            return Err(Error::Argument);
        }
        let producer = trace.definition(local.local)?;
        let Rvalue::Use(Operand::Copy(origin), _) = producer.value else {
            return Err(Error::Argument);
        };
        if *origin != mir::Place::from(parameter)
            || body.local_decls[parameter].ty != expected
            || trace
                .assignments
                .iter()
                .any(|a| a.destination.local == parameter)
            || !trace.before(producer.location, aggregate.location)
            || previous.is_some_and(|p| !trace.before(p, producer.location))
            || !used.insert(producer.location)
        {
            return Err(Error::Argument);
        }
        previous = Some(producer.location);
        fields.push((local.local, producer.location));
    }
    let records: HashSet<_> = body
        .local_decls
        .iter_enumerated()
        .filter(|(_, d)| d.ty == payload.ty())
        .map(|(l, _)| l)
        .collect();
    if records != HashSet::from([record.local, argument.local]) {
        return Err(Error::Argument);
    }
    Ok(Matched {
        record: record.local,
        argument: argument.local,
        transfer,
        aggregate: aggregate.location,
        movement: movement.location,
        fields,
    })
}
