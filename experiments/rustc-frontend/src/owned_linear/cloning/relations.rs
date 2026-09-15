//! Complete normal correspondence with two separately authenticated owner chains.
use super::{
    super::{LinearError as Error, Result, flow, values},
    Loan, Owner, calls, loans,
    source::{Plan, Step},
};
use rustc_hir::HirId;
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::{self, TyCtxt},
};
use std::collections::HashSet;

pub(super) struct Matched<'tcx> {
    pub parameter: mir::Local,
    pub bindings: Vec<(Owner, HirId, mir::Local, mir::Location)>,
    pub loan: Loan<'tcx>,
    pub read: mir::Location,
    pub drops: [(Owner, mir::Place<'tcx>, mir::Location); 2],
    pub returning: mir::Location,
}
pub(super) fn validate<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
) -> Result<Matched<'tcx>> {
    if body.source.def_id() != plan.frame.owner.to_def_id() {
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
    let trace = flow::trace(body)?;
    if trace.calls.len() != 2
        || trace.drops.len() != 2
        || trace
            .assignments
            .iter()
            .any(|a| a.destination.local == parameter)
    {
        return Err(Error::Call);
    }
    let box_ty = plan.construction.result();
    let (constructed, argument) = calls::read(
        tcx,
        body,
        trace.calls[0].1,
        plan.construction.constructor(),
        plan.construction.arguments(),
        box_ty,
    )?;
    let (cloned, clone_argument) = calls::read(
        tcx,
        body,
        trace.calls[1].1,
        plan.cloning.definition(),
        plan.cloning.arguments(),
        box_ty,
    )?;
    if constructed == cloned
        || !trace.before(trace.calls[0].0, trace.calls[1].0)
        || trace
            .assignments
            .iter()
            .any(|a| a.destination.local == constructed || a.destination.local == cloned)
    {
        return Err(Error::Call);
    }
    let mut used = HashSet::new();
    calls::scalar(
        tcx,
        body,
        &trace,
        argument,
        parameter,
        trace.calls[0].0,
        &mut used,
    )?;
    let mut locals = HashSet::new();
    let mut current = [None, None];
    let mut bindings: Vec<(Owner, HirId, mir::Local, mir::Location)> = Vec::new();
    let mut previous = None;
    let mut loan = None;
    for &step in &plan.steps {
        let owner = step.owner();
        let (local, location) = match step {
            Step::Construct(_) => {
                if previous.is_some() || current[0].is_some() {
                    return Err(Error::MoveGraph);
                }
                (constructed, trace.calls[0].0)
            }
            Step::Clone(_) => {
                if current[1].is_some() || loan.is_some() {
                    return Err(Error::MoveGraph);
                }
                if bindings
                    .iter()
                    .rev()
                    .find(|(tag, _, _, _)| *tag == Owner::Original)
                    .map(|(_, binding, _, _)| *binding)
                    != Some(plan.cloning.binding())
                {
                    return Err(Error::SourceIdentity);
                }
                loan = Some(loans::read(
                    tcx,
                    body,
                    &trace,
                    loans::Site {
                        form: plan.cloning.form(),
                        owner: current[0].ok_or(Error::MoveGraph)?,
                        previous: previous.ok_or(Error::MoveGraph)?,
                        call: trace.calls[1].0,
                        argument: clone_argument,
                    },
                    &mut used,
                )?);
                (cloned, trace.calls[1].0)
            }
            Step::Move(tag, _) => {
                let source = mir::Place::from(current[tag.index()].ok_or(Error::MoveGraph)?);
                let mut outgoing = trace.assignments.iter().filter(|a| matches!(a.value, Rvalue::Use(Operand::Move(actual), _) if *actual == source));
                let edge = outgoing.next().ok_or(Error::MoveGraph)?;
                if outgoing.next().is_some()
                    || body.local_decls[edge.destination.local].ty != box_ty
                    || trace.definition(edge.destination.local)?.location != edge.location
                    || !used.insert(edge.location)
                {
                    return Err(Error::MoveGraph);
                }
                (edge.destination.local, edge.location)
            }
        };
        if !locals.insert(local) || previous.is_some_and(|p| !trace.before(p, location)) {
            return Err(Error::MoveGraph);
        }
        current[owner.index()] = Some(local);
        previous = Some(location);
        bindings.push((owner, step.binding(), local, location));
    }
    let mut declared_boxes = HashSet::new();
    for (local, declaration) in body.local_decls.iter_enumerated() {
        if matches!(declaration.ty.kind(), ty::Adt(def, _) if Some(def.did()) == tcx.lang_items().owned_box())
        {
            if declaration.ty != box_ty {
                return Err(Error::MoveGraph);
            }
            declared_boxes.insert(local);
        }
    }
    if declared_boxes != locals {
        return Err(Error::MoveGraph);
    }
    let selected = current[plan.read.index()].ok_or(Error::Read)?;
    let (cast, read) = values::scalar_read(tcx, body, &trace, selected, &mut used)?;
    if !previous.is_some_and(|p| trace.before(p, cast)) {
        return Err(Error::Read);
    }
    let last = plan.steps.last().ok_or(Error::Drop)?.owner();
    let other = match last {
        Owner::Original => Owner::Cloned,
        Owner::Cloned => Owner::Original,
    };
    let mut drops = Vec::new();
    let mut after = read;
    for (owner, &(location, place)) in [last, other].into_iter().zip(&trace.drops) {
        if place != mir::Place::from(current[owner.index()].ok_or(Error::Drop)?)
            || !trace.before(after, location)
        {
            return Err(Error::Drop);
        }
        drops.push((owner, place, location));
        after = location;
    }
    super::super::multiple::residual::account(tcx, body, &trace, &mut used)?;
    if !trace.before(after, trace.returning) || used.len() != trace.assignments.len() {
        return Err(Error::Assignment);
    }
    Ok(Matched {
        parameter,
        bindings,
        loan: loan.ok_or(Error::Call)?,
        read,
        drops: drops.try_into().map_err(|_| Error::Drop)?,
        returning: trace.returning,
    })
}
