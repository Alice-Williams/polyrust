//! Complete event accounting; equal types or drop counts never choose an owner.
use super::super::{LinearError as E, Result, flow, multiple::residual, values};
use super::{
    aggregate::{self, Context, Which},
    cleanup, constructors, events,
    source::Plan,
};
use rustc_hir::{HirId, def_id::LocalDefId};
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::{self, TyCtxt},
};
use std::collections::{HashMap, HashSet};

pub(super) struct Matched<'tcx> {
    pub constructions: Vec<events::Construction>,
    pub aggregates: [events::Aggregate<'tcx>; 2],
    pub movements: Vec<events::Movement<'tcx>>,
    pub bindings: Vec<(HirId, mir::Local)>,
    pub leaves: Vec<events::Leaf<'tcx>>,
    pub cleanup: Vec<events::Cleanup<'tcx>>,
    pub read: mir::Location,
    pub returning: mir::Location,
}
pub(super) fn validate<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
) -> Result<Matched<'tcx>> {
    if body.source.def_id() != owner.to_def_id() || plan.frame.owner != owner {
        return Err(E::Owner);
    }
    if body.phase != mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup) {
        return Err(E::Phase);
    }
    let trace = flow::trace(body)?;
    if trace.drops.len() != plan.cleanup.len()
        || trace
            .assignments
            .iter()
            .filter(|a| matches!(a.value, Rvalue::Aggregate(..)))
            .count()
            != 2
    {
        return Err(E::BodyShape);
    }
    let mut used = HashSet::new();
    let constructions = constructors::read(tcx, plan, body, &trace, &mut used)?;
    let mut bindings = HashMap::new();
    let mut storage = HashSet::new();
    for construction in &constructions {
        if bindings
            .insert(construction.binding.0, construction.binding.1)
            .is_some()
            || !storage.insert(construction.binding.1)
        {
            return Err(E::MoveGraph);
        }
    }
    let previous = constructions.last().ok_or(E::BodyShape)?.call;
    let mut context = Context {
        tcx,
        body,
        trace: &trace,
        bindings: &mut bindings,
        storage: &mut storage,
        used: &mut used,
    };
    let inner = aggregate::read(&mut context, plan, Which::Inner, previous)?;
    let outer = aggregate::read(&mut context, plan, Which::Outer, inner.location)?;
    let mut previous = outer.location;
    let mut movements = Vec::new();
    for source in &plan.movements {
        let root = *bindings
            .get(&source.source.binding())
            .ok_or(E::SourceIdentity)?;
        let actual = source.source.project(tcx, body, root)?;
        let mut outgoing = trace.assignments.iter().filter(
            |a| matches!(a.value, Rvalue::Use(Operand::Move(place), _) if *place == actual),
        );
        let assignment = outgoing.next().ok_or(E::MoveGraph)?;
        if outgoing.next().is_some()
            || !assignment.destination.projection.is_empty()
            || actual.ty(&body.local_decls, tcx).ty != source.ty
            || assignment.destination.ty(&body.local_decls, tcx).ty != source.ty
            || !trace.before(previous, assignment.location)
            || !used.insert(assignment.location)
            || !storage.insert(assignment.destination.local)
            || bindings
                .insert(source.destination, assignment.destination.local)
                .is_some()
        {
            return Err(E::MoveGraph);
        }
        movements.push(events::Movement {
            source: source.source.clone(),
            actual,
            destination: (source.destination, assignment.destination.local),
            location: assignment.location,
        });
        previous = assignment.location;
    }
    let expected_types = [
        plan.constructions[0].input.result(),
        plan.inner.1.result(),
        plan.outer.1.result(),
    ];
    let mut actual_storage = HashSet::new();
    for (local, declaration) in body.local_decls.iter_enumerated() {
        if expected_types.contains(&declaration.ty) {
            actual_storage.insert(local);
        } else if declaration
            .ty
            .needs_drop(tcx, ty::TypingEnv::fully_monomorphized())
        {
            return Err(E::MoveGraph);
        }
    }
    if actual_storage != storage || bindings.len() != plan.frame.declarations.len() {
        return Err(E::MoveGraph);
    }
    let read_owner = *bindings.get(&plan.read).ok_or(E::Read)?;
    let (cast, read) = values::scalar_read(tcx, body, &trace, read_owner, &mut used)?;
    if !trace.before(previous, cast) {
        return Err(E::Read);
    }
    let (leaves, cleanup) = cleanup::read(tcx, plan, body, &trace, &bindings, read)?;
    residual::account_units(tcx, body, &trace, &mut used)?;
    residual::account(tcx, body, &trace, &mut used)?;
    if used.len() != trace.assignments.len() {
        return Err(E::Assignment);
    }
    let bindings = plan
        .frame
        .declarations
        .iter()
        .map(|(id, _)| {
            bindings
                .get(id)
                .map(|local| (*id, *local))
                .ok_or(E::SourceIdentity)
        })
        .collect::<Result<_>>()?;
    Ok(Matched {
        constructions,
        aggregates: [inner, outer],
        movements,
        bindings,
        leaves,
        cleanup,
        read,
        returning: trace.returning,
    })
}
