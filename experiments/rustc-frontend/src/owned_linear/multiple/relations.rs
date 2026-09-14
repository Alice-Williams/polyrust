//! Complete disjoint chains anchored by actual MIR scalar parameter producers.
use super::{
    super::{LinearError as Error, Result, flow, values},
    source::Plan,
};
use rustc_hir::{HirId, def_id::LocalDefId};
use rustc_middle::{
    mir::{self, Operand, Rvalue, TerminatorKind},
    ty::{self, TyCtxt},
};
use std::collections::{HashMap, HashSet};

pub(super) struct ChainMatched {
    pub parameter: mir::Local,
    pub owners: Vec<mir::Local>,
    pub moves: Vec<mir::Location>,
    pub drop: mir::Location,
}
pub(super) struct Matched {
    pub chains: Vec<ChainMatched>,
    pub read: mir::Location,
    pub drops: Vec<HirId>,
    pub returning: mir::Location,
}

pub(super) fn validate<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
) -> Result<Matched> {
    let trace = flow::trace(body)?;
    validate_path(tcx, owner, plan, body, &trace, HashSet::new())
}

pub(super) fn validate_path<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &flow::Trace<'_, 'tcx>,
    mut used: HashSet<mir::Location>,
) -> Result<Matched> {
    if body.source.def_id() != owner.to_def_id() {
        return Err(Error::Owner);
    }
    if body.phase != mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup) {
        return Err(Error::Phase);
    }
    let parameters: Vec<_> = body.args_iter().collect();
    if parameters.len() != plan.parameters.len()
        || body.local_decls[mir::RETURN_PLACE].ty != tcx.types.i32
        || parameters.len() != plan.argument_types.len()
        || parameters
            .iter()
            .zip(&plan.argument_types)
            .any(|(local, expected)| body.local_decls[*local].ty != *expected)
    {
        return Err(Error::Signature);
    }
    let parameter_map: HashMap<_, _> = plan
        .parameters
        .iter()
        .copied()
        .zip(parameters.iter().copied())
        .collect();
    if trace.calls.len() != plan.chains.len() || trace.drops.len() != plan.chains.len() {
        return Err(Error::Call);
    }
    let mut roots = HashMap::new();
    let mut call_anchors = Vec::new();
    for &(location, call) in &trace.calls {
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
        let parameter = values::argument_source(
            tcx,
            body,
            trace,
            &args[0].node,
            location,
            &parameters,
            &mut used,
        )?;
        let mut matching = plan
            .chains
            .iter()
            .filter(|chain| parameter_map.get(&chain.parameter) == Some(&parameter));
        let chain = matching.next().ok_or(Error::Argument)?;
        if matching.next().is_some() {
            return Err(Error::Ambiguous);
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
        if roots
            .insert(parameter, (destination.local, location))
            .is_some()
        {
            return Err(Error::Ambiguous);
        }
        call_anchors.push(chain.parameter);
    }
    // Constructor evaluation order is source statement order, not an arbitrary
    // local-number or graph traversal tie-breaker for ambiguous identities.
    if !call_anchors
        .iter()
        .copied()
        .eq(plan.chains.iter().map(|chain| chain.parameter))
    {
        return Err(Error::Call);
    }
    let box_ty = plan
        .chains
        .first()
        .ok_or(Error::BodyShape)?
        .constructor
        .result();
    if plan
        .chains
        .iter()
        .any(|chain| chain.constructor.result() != box_ty)
    {
        return Err(Error::MoveGraph);
    }
    let mut moves = Vec::new();
    for assignment in &trace.assignments {
        if let Rvalue::Use(Operand::Move(source), _) = assignment.value
            && source.ty(&body.local_decls, tcx).ty == box_ty
            && assignment.destination.ty(&body.local_decls, tcx).ty == box_ty
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
    if moves.len() + plan.chains.len() != plan.scopes.bindings().len() {
        return Err(Error::MoveGraph);
    }
    let mut all_owners = HashSet::new();
    let mut matched = Vec::new();
    let mut last_locations = Vec::new();
    let mut ends = HashMap::new();
    for chain in &plan.chains {
        let parameter = *parameter_map.get(&chain.parameter).ok_or(Error::Argument)?;
        let &(mut current, mut previous) = roots.get(&parameter).ok_or(Error::Call)?;
        if !all_owners.insert(current) {
            return Err(Error::MoveGraph);
        }
        let mut owners = vec![current];
        let mut locations = Vec::new();
        for _ in chain.bindings.iter().skip(1) {
            let mut outgoing = moves.iter().filter(|edge| edge.0 == current);
            let &(_, next, location) = outgoing.next().ok_or(Error::MoveGraph)?;
            if outgoing.next().is_some()
                || !all_owners.insert(next)
                || !trace.before(previous, location)
                || !used.insert(location)
            {
                return Err(Error::MoveGraph);
            }
            current = next;
            previous = location;
            owners.push(current);
            locations.push(location);
        }
        let mut drops = trace
            .drops
            .iter()
            .filter(|(_, place)| *place == mir::Place::from(current));
        let &(drop, _) = drops.next().ok_or(Error::Drop)?;
        if drops.next().is_some() {
            return Err(Error::Drop);
        }
        let last = *chain.bindings.last().ok_or(Error::MoveGraph)?;
        if ends.insert(last, current).is_some() {
            return Err(Error::MoveGraph);
        }
        matched.push(ChainMatched {
            parameter,
            owners,
            moves: locations,
            drop,
        });
        last_locations.push(previous);
    }
    let mut actual_owners = HashSet::new();
    for (local, declaration) in body.local_decls.iter_enumerated() {
        if matches!(declaration.ty.kind(), ty::Adt(definition, _) if Some(definition.did()) == tcx.lang_items().owned_box())
        {
            if declaration.ty != box_ty {
                return Err(Error::MoveGraph);
            }
            actual_owners.insert(local);
        }
    }
    if all_owners != actual_owners {
        return Err(Error::MoveGraph);
    }
    let read_owner = *ends.get(&plan.read).ok_or(Error::Read)?;
    let (cast, read) = values::scalar_read(tcx, body, trace, read_owner, &mut used)?;
    if last_locations
        .iter()
        .any(|&location| !trace.before(location, cast))
    {
        return Err(Error::Read);
    }
    let drop_order: Vec<_> = plan
        .scopes
        .bindings()
        .iter()
        .rev()
        .filter_map(|(id, _)| ends.contains_key(id).then_some(*id))
        .collect();
    if drop_order.len() != trace.drops.len() {
        return Err(Error::Drop);
    }
    for (&binding, &(location, place)) in drop_order.iter().zip(&trace.drops) {
        if place != mir::Place::from(ends[&binding])
            || !trace.before(read, location)
            || !trace.before(location, trace.returning)
        {
            return Err(Error::Drop);
        }
    }
    super::residual::account(tcx, body, trace, &mut used)?;
    if used.len() != trace.assignments.len() {
        return Err(Error::Assignment);
    }
    Ok(Matched {
        chains: matched,
        read,
        drops: drop_order,
        returning: trace.returning,
    })
}
