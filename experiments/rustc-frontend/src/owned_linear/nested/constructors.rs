//! Authenticate constructor identity and distinct parameter-rooted scalar staging.
use super::super::{LinearError as E, Result, flow, values};
use super::{events, source::Plan};
use rustc_middle::{
    mir::{self, Operand, TerminatorKind},
    ty::{self, TyCtxt},
};
use std::collections::HashSet;

pub(super) fn read<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &flow::Trace<'_, 'tcx>,
    used: &mut HashSet<mir::Location>,
) -> Result<Vec<events::Construction>> {
    let parameters: Vec<_> = body.args_iter().collect();
    if parameters.len() != 3
        || trace.calls.len() != 3
        || body.local_decls[mir::RETURN_PLACE].ty != tcx.types.i32
        || parameters
            .iter()
            .any(|p| body.local_decls[*p].ty != tcx.types.i32)
    {
        return Err(E::Signature);
    }
    let mut result = Vec::new();
    let mut previous = None;
    for (source, &(location, call)) in plan.constructions.iter().zip(&trace.calls) {
        let TerminatorKind::Call {
            func: func @ Operand::Constant(_),
            args,
            destination,
            ..
        } = call
        else {
            return Err(E::Call);
        };
        let ty::FnDef(definition, arguments) = *func.ty(&body.local_decls, tcx).kind() else {
            return Err(E::Call);
        };
        if definition != source.input.constructor()
            || arguments != source.input.arguments()
            || args.len() != 1
            || !destination.projection.is_empty()
            || destination.ty(&body.local_decls, tcx).ty != source.input.result()
        {
            return Err(E::Call);
        }
        let before = used.clone();
        let parameter =
            values::argument_source(tcx, body, trace, &args[0].node, location, &parameters, used)?;
        let expected = plan
            .frame
            .parameters
            .iter()
            .position(|p| *p == source.parameter)
            .ok_or(E::Argument)?;
        if parameter != parameters[expected] {
            return Err(E::Argument);
        }
        let staging: Vec<_> = trace
            .assignments
            .iter()
            .filter(|a| used.contains(&a.location) && !before.contains(&a.location))
            .map(|a| a.location)
            .collect();
        if previous.is_some_and(|p| {
            !trace.before(p, location) || staging.iter().any(|s| !trace.before(p, *s))
        }) {
            return Err(E::Argument);
        }
        previous = Some(location);
        result.push(events::Construction {
            parameter: (source.parameter, parameter),
            binding: (source.binding, destination.local),
            call: location,
            staging,
        });
    }
    Ok(result)
}
