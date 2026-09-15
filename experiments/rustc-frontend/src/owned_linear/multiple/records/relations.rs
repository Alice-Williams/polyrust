//! Correlate source paths with complete compiler places, never debug variables.
use super::{
    super::super::{LinearError as Error, Result, flow, values},
    Transfer, aggregate, constructors,
    source::{Plan, SourcePlace},
};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::{self, TyCtxt},
};
use std::collections::{HashMap, HashSet};

pub(super) struct ChainMatched<'tcx> {
    pub parameter: mir::Local,
    pub places: Vec<mir::Place<'tcx>>,
    pub transfers: Vec<Transfer<'tcx>>,
    pub drop: mir::Location,
}
pub(super) struct Matched<'tcx> {
    pub chains: Vec<ChainMatched<'tcx>>,
    pub aggregate: (mir::Place<'tcx>, mir::Location),
    pub read: mir::Location,
    pub drops: Vec<SourcePlace>,
}
pub(super) fn validate<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
) -> Result<Matched<'tcx>> {
    if body.source.def_id() != owner.to_def_id() {
        return Err(Error::Owner);
    }
    if body.phase != mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup) {
        return Err(Error::Phase);
    }
    let trace = flow::trace(body)?;
    let aggregate = aggregate::read(tcx, plan, body, &trace)?;
    let mut used = HashSet::from([aggregate.location]);
    let roots = constructors::read(tcx, plan, body, &trace, &mut used)?;
    let box_ty = plan
        .chains
        .first()
        .ok_or(Error::BodyShape)?
        .constructor
        .result();
    let mut all_places = HashSet::new();
    let mut all_locals = HashSet::new();
    let mut matched = Vec::new();
    let mut ends = HashMap::new();
    let mut binding_events = HashMap::from([(plan.binding, aggregate.location)]);
    let mut last_events = Vec::new();
    for (chain, (parameter, mut current, mut previous)) in plan.chains.iter().zip(roots) {
        if chain.constructor.result() != box_ty
            || !all_places.insert(current)
            || !all_locals.insert(current.local)
        {
            return Err(Error::MoveGraph);
        }
        let Some(&SourcePlace::Local(first)) = chain.places.first() else {
            return Err(Error::MoveGraph);
        };
        if binding_events.insert(first, previous).is_some() {
            return Err(Error::SourceIdentity);
        }
        let mut places = vec![current];
        let mut transfers = Vec::new();
        for step in chain.places.iter().skip(1) {
            match *step {
                SourcePlace::Field {
                    record,
                    declaration,
                    index,
                } => {
                    let field = aggregate
                        .fields
                        .iter()
                        .find(|f| f.index == index)
                        .ok_or(Error::MoveGraph)?;
                    let declared = plan
                        .record
                        .fields()
                        .iter()
                        .find(|f| f.index() == index)
                        .ok_or(Error::SourceIdentity)?;
                    if record != plan.binding
                        || declared.declaration() != declaration
                        || current != field.source
                        || !trace.before(previous, field.movement)
                        || !all_locals.insert(field.staging.local)
                        || !all_places.insert(field.staging)
                        || !all_places.insert(field.place)
                        || !used.insert(field.movement)
                    {
                        return Err(Error::MoveGraph);
                    }
                    current = field.place;
                    previous = aggregate.location;
                    transfers.push(Transfer::IntoField {
                        staging: field.staging,
                        movement: field.movement,
                        aggregate: aggregate.location,
                    });
                }
                SourcePlace::Local(binding) => {
                    let mut outgoing = trace.assignments.iter().filter(|a|
                        matches!(a.value, Rvalue::Use(Operand::Move(source), _) if *source == current));
                    let next = outgoing.next().ok_or(Error::MoveGraph)?;
                    if outgoing.next().is_some()
                        || next.destination.ty(&body.local_decls, tcx).ty != box_ty
                        || !trace.before(previous, next.location)
                        || !used.insert(next.location)
                        || !all_places.insert(next.destination)
                        || !all_locals.insert(next.destination.local)
                        || binding_events.insert(binding, next.location).is_some()
                    {
                        return Err(Error::MoveGraph);
                    }
                    current = next.destination;
                    previous = next.location;
                    transfers.push(Transfer::Move {
                        location: next.location,
                    });
                }
            }
            places.push(current);
        }
        let mut drops = trace.drops.iter().filter(|(_, place)| *place == current);
        let &(drop, _) = drops.next().ok_or(Error::Drop)?;
        if drops.next().is_some()
            || ends
                .insert(*chain.places.last().unwrap(), current)
                .is_some()
        {
            return Err(Error::Drop);
        }
        matched.push(ChainMatched {
            parameter,
            places,
            transfers,
            drop,
        });
        last_events.push(previous);
    }
    // Record-field projections share one root, but all other Box storage must be
    // exactly a source owner or an authenticated initializer staging temporary.
    let mut actual_locals = HashSet::new();
    let mut records = Vec::new();
    for (local, declaration) in body.local_decls.iter_enumerated() {
        if declaration.ty == plan.record.result() {
            records.push(mir::Place::from(local));
        }
        if matches!(declaration.ty.kind(), ty::Adt(def, _) if Some(def.did()) == tcx.lang_items().owned_box())
        {
            if declaration.ty != box_ty {
                return Err(Error::MoveGraph);
            }
            actual_locals.insert(local);
        }
    }
    if actual_locals != all_locals || records != [aggregate.destination] {
        return Err(Error::MoveGraph);
    }
    let mut previous = None;
    for &(binding, _) in plan.scopes.bindings() {
        let event = *binding_events.get(&binding).ok_or(Error::Scope)?;
        if previous.is_some_and(|p| !trace.before(p, event)) {
            return Err(Error::MoveGraph);
        }
        if binding == plan.binding
            && previous.is_some_and(|p| {
                aggregate
                    .fields
                    .iter()
                    .any(|f| !trace.before(p, f.movement))
            })
        {
            return Err(Error::MoveGraph);
        }
        previous = Some(event);
    }
    if binding_events.len() != plan.scopes.bindings().len() {
        return Err(Error::Scope);
    }
    let read_owner = ends
        .get(&SourcePlace::Local(plan.read))
        .ok_or(Error::Read)?;
    if !read_owner.projection.is_empty() {
        return Err(Error::Read);
    }
    let (cast, read) = values::scalar_read(tcx, body, &trace, read_owner.local, &mut used)?;
    if last_events.iter().any(|event| !trace.before(*event, cast)) {
        return Err(Error::Read);
    }
    let drops = drop_order(plan, &ends);
    if drops.len() != trace.drops.len() || drops.len() != plan.chains.len() {
        return Err(Error::Drop);
    }
    for (source, &(location, place)) in drops.iter().zip(&trace.drops) {
        if ends.get(source) != Some(&place)
            || !trace.before(read, location)
            || !trace.before(location, trace.returning)
        {
            return Err(Error::Drop);
        }
    }
    super::super::residual::account_units(tcx, body, &trace, &mut used)?;
    super::super::residual::account(tcx, body, &trace, &mut used)?;
    if used.len() != trace.assignments.len() {
        return Err(Error::Assignment);
    }
    Ok(Matched {
        chains: matched,
        aggregate: (aggregate.destination, aggregate.location),
        read,
        drops,
    })
}

fn drop_order(plan: &Plan<'_>, ends: &HashMap<SourcePlace, mir::Place<'_>>) -> Vec<SourcePlace> {
    let mut result = Vec::new();
    for &(binding, _) in plan.scopes.bindings().iter().rev() {
        if binding == plan.binding {
            for (index, field) in plan
                .record
                .definition()
                .non_enum_variant()
                .fields
                .iter_enumerated()
            {
                let place = SourcePlace::Field {
                    record: binding,
                    declaration: field.did,
                    index,
                };
                if ends.contains_key(&place) {
                    result.push(place);
                }
            }
        } else if ends.contains_key(&SourcePlace::Local(binding)) {
            result.push(SourcePlace::Local(binding));
        }
    }
    result
}
