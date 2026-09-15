//! Account for the entire normal trace, not just a matching constructor call.
use super::super::{
    super::{LinearError as Error, Result, flow, values},
    residual,
};
use super::{aggregate, source::Plan};
use rustc_middle::{
    mir::{self, Operand, ProjectionElem, Rvalue, TerminatorKind},
    ty::{self, TyCtxt},
};
use std::collections::HashSet;

pub(super) struct Matched {
    pub payload: aggregate::Matched,
    pub owners: Vec<mir::Local>,
    pub moves: Vec<mir::Location>,
    pub pointer: mir::Local,
    pub read: mir::Location,
    pub drop: mir::Location,
    pub returning: mir::Location,
}
pub(super) fn validate<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
) -> Result<Matched> {
    if body.source.def_id() != plan.constructor.owner().to_def_id() {
        return Err(Error::Owner);
    }
    if body.phase != mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup) {
        return Err(Error::Phase);
    }
    super::source::check_scope(tcx, plan)?;
    let selected = &plan.constructor.payload().fields()[plan.selected.as_usize()];
    if body.arg_count != plan.parameters.len()
        || body.local_decls[mir::RETURN_PLACE].ty != selected.ty()
        || body
            .args_iter()
            .zip(&plan.parameters)
            .any(|(l, (_, ty))| body.local_decls[l].ty != *ty)
    {
        return Err(Error::Signature);
    }
    let flow = flow::read(body)?;
    let TerminatorKind::Call {
        func,
        args,
        destination,
        ..
    } = flow.call.1
    else {
        return Err(Error::Call);
    };
    let ty::FnDef(definition, arguments) = *func.ty(&body.local_decls, tcx).kind() else {
        return Err(Error::Call);
    };
    if definition != plan.constructor.constructor()
        || arguments != plan.constructor.arguments()
        || !destination.projection.is_empty()
        || destination.ty(&body.local_decls, tcx).ty != plan.constructor.result()
        || args.len() != 1
    {
        return Err(Error::Call);
    }
    let mut used = HashSet::new();
    let payload = aggregate::read(
        tcx,
        plan,
        body,
        &flow,
        &args[0].node,
        flow.call.0,
        &mut used,
    )?;
    let mut owners = vec![destination.local];
    let mut unique = HashSet::from([destination.local]);
    let mut moves = Vec::new();
    let mut previous = flow.call.0;
    for _ in plan.owners.iter().skip(1) {
        let current = *owners.last().unwrap();
        let mut matching = flow.assignments.iter().filter(|a| {
            matches!(a.value, Rvalue::Use(Operand::Move(p), _) if *p == mir::Place::from(current))
        });
        let next = matching.next().ok_or(Error::MoveGraph)?;
        if matching.next().is_some()
            || next.destination.ty(&body.local_decls, tcx).ty != plan.constructor.result()
            || !unique.insert(next.destination.local)
            || !flow.before(previous, next.location)
            || !used.insert(next.location)
        {
            return Err(Error::MoveGraph);
        }
        owners.push(next.destination.local);
        moves.push(next.location);
        previous = next.location;
    }
    let mut all_boxes = HashSet::new();
    for (local, declaration) in body.local_decls.iter_enumerated() {
        if matches!(declaration.ty.kind(), ty::Adt(def, _) if Some(def.did()) == tcx.lang_items().owned_box())
        {
            if declaration.ty != plan.constructor.result() {
                return Err(Error::MoveGraph);
            }
            all_boxes.insert(local);
        }
    }
    if all_boxes != unique
        || flow
            .assignments
            .iter()
            .any(|a| a.destination.local == destination.local)
    {
        return Err(Error::MoveGraph);
    }
    let current = *owners.last().unwrap();
    let read = flow.definition(mir::RETURN_PLACE)?;
    let Rvalue::Use(Operand::Copy(place), _) = read.value else {
        return Err(Error::Read);
    };
    if !matches!(place.projection.as_slice(), [ProjectionElem::Deref, ProjectionElem::Field(index, ty)] if *index == plan.selected && *ty == selected.ty())
    {
        return Err(Error::Read);
    }
    let cast = values::payload_pointer(
        tcx,
        body,
        &flow,
        current,
        place.local,
        plan.constructor.payload().ty(),
    )?;
    if !flow.before(previous, cast)
        || !flow.before(cast, read.location)
        || !used.insert(cast)
        || !used.insert(read.location)
    {
        return Err(Error::Read);
    }
    if flow.drop.1 != mir::Place::from(current)
        || !flow.before(read.location, flow.drop.0)
        || !flow.before(flow.drop.0, flow.returning)
    {
        return Err(Error::Drop);
    }
    residual::account_units(tcx, body, &flow, &mut used)?;
    residual::account(tcx, body, &flow, &mut used)?;
    if used.len() != flow.assignments.len() {
        return Err(Error::Assignment);
    }
    Ok(Matched {
        payload,
        owners,
        moves,
        pointer: place.local,
        read: read.location,
        drop: flow.drop.0,
        returning: flow.returning,
    })
}
