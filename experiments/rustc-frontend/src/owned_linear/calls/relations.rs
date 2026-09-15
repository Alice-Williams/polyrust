//! Complete ordered body relation shared by the closed entry and leaf grammars.
use super::super::{LinearError as Error, Result, flow, multiple::residual, values};
use super::evidence::{BodyEnding as Ending, BodyStep as Step, CallSite};
use super::{
    moves::Owners,
    source::{Finish, Operation, Plan},
};
use rustc_hir::HirId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, TerminatorKind},
    ty::{self, TyCtxt},
};
use std::collections::HashSet;
pub(super) struct Matched {
    pub parameter: mir::Local,
    pub bindings: Vec<(HirId, mir::Local)>,
    pub steps: Vec<Step>,
    pub ending: Ending,
    pub returning: mir::Location,
}
pub(super) fn validate<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
) -> Result<Matched> {
    if body.source.def_id() != plan.frame.owner.to_def_id() {
        return Err(Error::Owner);
    }
    if body.phase != mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup) {
        return Err(Error::Phase);
    }
    if body.arg_count != 1
        || body.local_decls[mir::RETURN_PLACE].ty != plan.frame.signature.output()
    {
        return Err(Error::Signature);
    }
    let parameter = body.args_iter().next().ok_or(Error::Signature)?;
    if body.local_decls[parameter].ty != plan.frame.signature.inputs()[0] {
        return Err(Error::Signature);
    }
    let trace = flow::trace(body)?;
    if trace
        .assignments
        .iter()
        .any(|a| a.destination.local == parameter)
    {
        return Err(Error::Assignment);
    }
    let mut owners =
        Owners::new((body.local_decls[parameter].ty == plan.box_ty).then_some(parameter));
    let mut used = HashSet::new();
    let mut calls = trace.calls.iter();
    let mut steps = Vec::new();
    let mut bindings = Vec::new();
    let mut last = None;
    for operation in &plan.operations {
        if let Operation::Move(binding) = operation {
            let (local, location) = owners.advance(body, &trace, plan.box_ty, &mut used)?;
            if local == mir::RETURN_PLACE {
                return Err(Error::MoveGraph);
            }
            bindings.push((*binding, local));
            steps.push(Step::Move(location));
            last = Some(location);
            continue;
        }
        let (definition, arguments, input_ty, output_ty, binding) = match operation {
            Operation::Allocate { input, binding } => (
                input.constructor(),
                input.arguments(),
                tcx.types.i32,
                input.result(),
                *binding,
            ),
            Operation::Call { input, binding } => (
                input.definition(),
                input.arguments(),
                input.signature().inputs()[0],
                input.signature().output(),
                *binding,
            ),
            Operation::Move(_) => unreachable!(),
        };
        let (
            at,
            TerminatorKind::Call {
                func: Operand::Constant(func),
                args,
                destination,
                ..
            },
        ) = calls.next().ok_or(Error::Call)?
        else {
            return Err(Error::Call);
        };
        if !matches!(func.const_.ty().kind(), ty::FnDef(def, arguments_) if *def == definition && *arguments_ == arguments)
            || args.len() != 1
            || !destination.projection.is_empty()
            || destination.ty(&body.local_decls, tcx).ty != output_ty
            || (binding.is_none() != (destination.local == mir::RETURN_PLACE))
            || trace
                .assignments
                .iter()
                .any(|a| a.destination.local == destination.local)
            || last.is_some_and(|p| !trace.before(p, *at))
        {
            return Err(Error::Call);
        }
        let before = used.clone();
        let argument = if input_ty == tcx.types.i32 {
            if owners.current.is_some() {
                return Err(Error::MoveGraph);
            }
            values::argument_source(
                tcx,
                body,
                &trace,
                &args[0].node,
                *at,
                &[parameter],
                &mut used,
            )?;
            let (Operand::Move(place) | Operand::Copy(place)) = args[0].node else {
                return Err(Error::Argument);
            };
            place.local
        } else {
            let (staged, location) = owners.advance(body, &trace, plan.box_ty, &mut used)?;
            if staged == mir::RETURN_PLACE
                || args[0].node != Operand::Move(mir::Place::from(staged))
                || !trace.before(location, *at)
            {
                return Err(Error::Argument);
            }
            owners.current = None;
            staged
        };
        let staging: Vec<_> = trace
            .assignments
            .iter()
            .filter(|a| used.contains(&a.location) && !before.contains(&a.location))
            .map(|a| a.location)
            .collect();
        if argument == parameter || staging.is_empty() {
            return Err(Error::Argument);
        }
        if input_ty == tcx.types.i32 {
            let copied = trace.definition(argument)?;
            if staging != [copied.location]
                || !matches!(copied.value, Rvalue::Use(Operand::Copy(place), _) if *place == mir::Place::from(parameter))
            {
                return Err(Error::Argument);
            }
        }
        if output_ty == plan.box_ty {
            owners.receive(destination.local, *at)?;
        }
        if let Some(binding) = binding {
            bindings.push((binding, destination.local));
        }
        let site = CallSite {
            location: *at,
            argument,
            destination: destination.local,
            staging,
        };
        steps.push(match operation {
            Operation::Allocate { .. } => Step::Allocation(site),
            _ => Step::LocalCall(site),
        });
        last = Some(*at);
    }
    if calls.next().is_some() {
        return Err(Error::Call);
    }
    let ending = match plan.finish {
        Finish::Read => {
            let (owner, produced) = owners.current.ok_or(Error::Read)?;
            let (cast, read) = values::scalar_read(tcx, body, &trace, owner, &mut used)?;
            if produced.is_some_and(|p| !trace.before(p, cast)) {
                return Err(Error::Read);
            }
            let [(drop, place)] = trace.drops.as_slice() else {
                return Err(Error::Drop);
            };
            if *place != mir::Place::from(owner)
                || !trace.before(read, *drop)
                || !trace.before(*drop, trace.returning)
            {
                return Err(Error::Drop);
            }
            Ending::ReadDrop {
                cast,
                read,
                drop: *drop,
            }
        }
        Finish::Owner => {
            let (returned, at) = owners.advance(body, &trace, plan.box_ty, &mut used)?;
            if returned != mir::RETURN_PLACE
                || !trace.drops.is_empty()
                || !trace.before(at, trace.returning)
            {
                return Err(Error::MoveGraph);
            }
            Ending::OwnerReturn(at)
        }
        Finish::Direct => {
            if !trace.drops.is_empty()
                || !last.is_some_and(|p| trace.before(p, trace.returning))
                || (plan.frame.signature.output() == plan.box_ty
                    && owners.current.map(|p| p.0) != Some(mir::RETURN_PLACE))
                || (plan.frame.signature.output() == tcx.types.i32 && owners.current.is_some())
            {
                return Err(Error::MoveGraph);
            }
            Ending::DirectReturn
        }
    };
    owners.finish(tcx, body, plan.box_ty)?;
    residual::account_units(tcx, body, &trace, &mut used)?;
    residual::account(tcx, body, &trace, &mut used)?;
    if used.len() != trace.assignments.len() {
        return Err(Error::Assignment);
    }
    Ok(Matched {
        parameter,
        bindings,
        steps,
        ending,
        returning: trace.returning,
    })
}
