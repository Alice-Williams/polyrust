//! Complete one-to-one producer/consumer relation for the closed source shape.
use super::{LinearError as Error, Result, flow, source::Plan, values};
use rustc_middle::{
    mir::{self, Operand, Rvalue, TerminatorKind},
    ty::{self, TyCtxt},
};
use std::collections::HashSet;

pub(super) struct Matched {
    pub parameter: mir::Local,
    pub owners: Vec<mir::Local>,
    pub moves: Vec<mir::Location>,
    pub scalar_read: mir::Location,
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
    if body.arg_count != 1 || body.local_decls[mir::RETURN_PLACE].ty != tcx.types.i32 {
        return Err(Error::Signature);
    }
    let parameter = body.args_iter().next().ok_or(Error::Signature)?;
    if body.local_decls[parameter].ty != tcx.types.i32 {
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
    let ty::FnDef(def, arguments) = *func.ty(&body.local_decls, tcx).kind() else {
        return Err(Error::Call);
    };
    if def != plan.constructor.constructor()
        || arguments != plan.constructor.arguments()
        || !destination.projection.is_empty()
        || destination.ty(&body.local_decls, tcx).ty != plan.constructor.result()
        || args.len() != 1
    {
        return Err(Error::Call);
    }
    let mut used = HashSet::new();
    values::argument(tcx, body, &flow, &args[0].node, parameter, &mut used)?;
    let mut moves = Vec::new();
    for assignment in &flow.assignments {
        let Rvalue::Use(Operand::Move(source), _) = assignment.value else {
            continue;
        };
        if source.ty(&body.local_decls, tcx).ty == plan.constructor.result()
            && assignment.destination.ty(&body.local_decls, tcx).ty == plan.constructor.result()
        {
            if !source.projection.is_empty() {
                return Err(Error::MoveGraph);
            }
            moves.push((
                source.local,
                assignment.destination.local,
                assignment.location,
            ));
        }
    }
    if moves.len() + 1 != plan.bindings.len() {
        return Err(Error::MoveGraph);
    }
    let mut current = destination.local;
    let mut owners = vec![current];
    let mut unique = HashSet::from([current]);
    let mut previous = flow.call.0;
    let mut move_locations = Vec::new();
    for _ in plan.bindings.iter().skip(1) {
        let mut outgoing = moves.iter().filter(|edge| edge.0 == current);
        let &(_, next, location) = outgoing.next().ok_or(Error::MoveGraph)?;
        if outgoing.next().is_some()
            || !unique.insert(next)
            || !flow.before(previous, location)
            || !used.insert(location)
        {
            return Err(Error::MoveGraph);
        }
        current = next;
        owners.push(current);
        move_locations.push(location);
        previous = location;
    }
    let mut all_box_locals = HashSet::new();
    for (local, declaration) in body.local_decls.iter_enumerated() {
        if matches!(declaration.ty.kind(), ty::Adt(def, _) if Some(def.did()) == tcx.lang_items().owned_box())
        {
            if declaration.ty != plan.constructor.result() {
                return Err(Error::MoveGraph);
            }
            all_box_locals.insert(local);
        }
    }
    if unique != all_box_locals {
        return Err(Error::MoveGraph);
    }
    let (cast, scalar_read) = values::scalar_read(tcx, body, &flow, current, &mut used)?;
    if !flow.before(previous, cast)
        || !flow.before(scalar_read, flow.drop.0)
        || !flow.before(flow.drop.0, flow.returning)
        || flow.drop.1.local != current
        || !flow.drop.1.projection.is_empty()
    {
        return Err(Error::Drop);
    }
    if used.len() != flow.assignments.len() {
        return Err(Error::Assignment);
    }
    Ok(Matched {
        parameter,
        owners,
        moves: move_locations,
        scalar_read,
        drop: flow.drop.0,
        returning: flow.returning,
    })
}
