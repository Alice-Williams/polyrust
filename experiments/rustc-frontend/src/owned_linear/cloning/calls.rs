//! Exact call constants and one scalar Copy staging edge for construction.
use super::super::{LinearError as Error, Result, flow::Trace};
use rustc_hir::def_id::DefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue, TerminatorKind},
    ty::{self, GenericArgsRef, Ty, TyCtxt},
};
use std::collections::HashSet;

pub(super) fn read<'a, 'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    call: &'a TerminatorKind<'tcx>,
    definition: DefId,
    arguments: GenericArgsRef<'tcx>,
    result: Ty<'tcx>,
) -> Result<(mir::Local, &'a Operand<'tcx>)> {
    let TerminatorKind::Call {
        func,
        args,
        destination,
        ..
    } = call
    else {
        return Err(Error::Call);
    };
    if !matches!(func, Operand::Constant(_))
        || args.len() != 1
        || !destination.projection.is_empty()
        || destination.ty(&body.local_decls, tcx).ty != result
    {
        return Err(Error::Call);
    }
    if !matches!(func.ty(&body.local_decls, tcx).kind(), ty::FnDef(actual, actual_args) if *actual == definition && *actual_args == arguments)
    {
        return Err(Error::Call);
    }
    Ok((destination.local, &args[0].node))
}
pub(super) fn scalar<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &Trace<'_, 'tcx>,
    argument: &Operand<'tcx>,
    parameter: mir::Local,
    call: mir::Location,
    used: &mut HashSet<mir::Location>,
) -> Result<()> {
    let Operand::Move(stage) = *argument else {
        return Err(Error::Argument);
    };
    if !stage.projection.is_empty()
        || stage.local == parameter
        || body.local_decls[stage.local].ty != tcx.types.i32
    {
        return Err(Error::Argument);
    }
    let definition = trace.definition(stage.local)?;
    if !matches!(definition.value, Rvalue::Use(Operand::Copy(source), _) if *source == mir::Place::from(parameter))
        || !trace.before(definition.location, call)
        || !used.insert(definition.location)
    {
        return Err(Error::Argument);
    }
    Ok(())
}
