//! Match actual leaf/whole-record Drop events without inventing field drops.
use super::super::{LinearError as E, Result, flow};
use super::{
    events::{self, CleanupKind},
    source::Plan,
};
use rustc_hir::HirId;
use rustc_middle::{mir, ty::TyCtxt};
use std::collections::{HashMap, HashSet};

pub(super) fn read<'tcx>(
    tcx: TyCtxt<'tcx>,
    plan: &Plan<'tcx>,
    body: &mir::Body<'tcx>,
    trace: &flow::Trace<'_, 'tcx>,
    bindings: &HashMap<HirId, mir::Local>,
    read: mir::Location,
) -> Result<(Vec<events::Leaf<'tcx>>, Vec<events::Cleanup<'tcx>>)> {
    if plan.cleanup.len() != trace.drops.len() {
        return Err(E::Drop);
    }
    let mut leaves = Vec::new();
    let mut cleanup = Vec::new();
    let mut covered = HashSet::new();
    for ((kind, source), &(location, actual)) in plan.cleanup.iter().zip(&trace.drops) {
        let root = *bindings.get(&source.binding()).ok_or(E::SourceIdentity)?;
        let expected = source.project(tcx, body, root)?;
        let (count, ty) = match kind {
            CleanupKind::Leaf => (1, plan.constructions[0].input.result()),
            CleanupKind::InnerRecord => (2, plan.inner.1.result()),
        };
        if actual != expected
            || actual.ty(&body.local_decls, tcx).ty != ty
            || !trace.before(read, location)
            || !trace.before(location, trace.returning)
        {
            return Err(E::Drop);
        }
        let mut constructors = Vec::new();
        for leaf in plan
            .leaves
            .iter()
            .filter(|leaf| source.contains(&leaf.current))
        {
            if !covered.insert(leaf.constructor) {
                return Err(E::Drop);
            }
            constructors.push(leaf.constructor);
            leaves.push(events::Leaf {
                constructor: leaf.constructor,
                source: leaf.current.clone(),
                actual: leaf.current.project(tcx, body, root)?,
                cleanup: location,
            });
        }
        if constructors.len() != count {
            return Err(E::Drop);
        }
        cleanup.push(events::Cleanup {
            kind: *kind,
            source: source.clone(),
            actual,
            location,
            constructors,
        });
    }
    if leaves.len() != 3 || covered != plan.constructions.iter().map(|c| c.binding).collect() {
        return Err(E::Drop);
    }
    Ok((leaves, cleanup))
}
