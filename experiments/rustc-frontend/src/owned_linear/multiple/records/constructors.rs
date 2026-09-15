//! Exact scalar producer anchors and compiler Box constructor identities.
use super::{
    super::super::{LinearError as Error, Result, flow, values},
    source::Plan,
};
use rustc_middle::{
    mir::{self, TerminatorKind},
    ty::{self, TyCtxt},
};
use std::collections::HashSet;

pub(super) fn read<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &flow::Trace<'_, 'tcx>,
    used: &mut HashSet<mir::Location>,
) -> Result<Vec<(mir::Local, mir::Place<'tcx>, mir::Location)>> {
    let parameters: Vec<_> = body.args_iter().collect();
    if parameters.len() != plan.parameters.len()
        || body.local_decls[mir::RETURN_PLACE].ty != tcx.types.i32
        || parameters
            .iter()
            .any(|p| body.local_decls[*p].ty != tcx.types.i32)
        || trace.calls.len() != plan.chains.len()
    {
        return Err(Error::Signature);
    }
    let mut roots = Vec::new();
    for (chain, &(location, call)) in plan.chains.iter().zip(&trace.calls) {
        let TerminatorKind::Call {
            func,
            args,
            destination,
            ..
        } = call
        else {
            return Err(Error::Call);
        };
        if args.len() != 1 || !destination.projection.is_empty() {
            return Err(Error::Call);
        }
        let parameter =
            values::argument_source(tcx, body, trace, &args[0].node, location, &parameters, used)?;
        let expected = plan
            .parameters
            .iter()
            .position(|p| *p == chain.parameter)
            .ok_or(Error::Argument)?;
        if parameter != parameters[expected] {
            return Err(Error::Argument);
        }
        let ty::FnDef(definition, arguments) = *func.ty(&body.local_decls, tcx).kind() else {
            return Err(Error::Call);
        };
        if definition != chain.constructor.constructor()
            || arguments != chain.constructor.arguments()
            || destination.ty(&body.local_decls, tcx).ty != chain.constructor.result()
        {
            return Err(Error::Call);
        }
        roots.push((parameter, *destination, location));
    }
    Ok(roots)
}
