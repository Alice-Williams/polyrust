//! Constant-defined cleanup decisions, independently checked against live owners.
use super::{Assignment, Trace};
use crate::owned_linear::{LinearError as Error, Result};
use rustc_middle::{
    mir::{
        self, Operand, Rvalue,
        visit::{MutatingUseContext, NonMutatingUseContext, NonUseContext, PlaceContext, Visitor},
    },
    ty::{self, TyCtxt},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Decision {
    pub(in crate::owned_linear) local: mir::Local,
    pub(in crate::owned_linear) definition: mir::Location,
    pub(in crate::owned_linear) location: mir::Location,
    pub(in crate::owned_linear) value: bool,
    pub(in crate::owned_linear) target: mir::BasicBlock,
}
impl Decision {
    pub(crate) fn local(&self) -> mir::Local {
        self.local
    }
    pub(crate) fn definition(&self) -> mir::Location {
        self.definition
    }
    pub(crate) fn location(&self) -> mir::Location {
        self.location
    }
    pub(crate) fn value(&self) -> bool {
        self.value
    }
    pub(crate) fn target(&self) -> mir::BasicBlock {
        self.target
    }
}

fn constant<'tcx>(tcx: TyCtxt<'tcx>, value: &Rvalue<'tcx>) -> Result<bool> {
    let Rvalue::Use(Operand::Constant(value), _) = value else {
        return Err(Error::Assignment);
    };
    if value.const_.ty() != tcx.types.bool {
        return Err(Error::Assignment);
    }
    value
        .const_
        .try_eval_bool(tcx, ty::TypingEnv::fully_monomorphized())
        .ok_or(Error::Assignment)
}

pub(super) fn resolve<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    assignments: &[Assignment<'_, 'tcx>],
    operand: &Operand<'tcx>,
    targets: &mir::SwitchTargets,
    location: mir::Location,
) -> Result<Decision> {
    let Operand::Copy(place) = operand else {
        return Err(Error::ControlFlow);
    };
    if !place.projection.is_empty()
        || body.local_decls[place.local].ty != tcx.types.bool
        || place.local.as_usize() <= body.arg_count
    {
        return Err(Error::ControlFlow);
    }
    let definition = assignments
        .iter()
        .rev()
        .find(|a| a.destination.local == place.local)
        .ok_or(Error::Assignment)?;
    let value = constant(tcx, definition.value)?;
    Ok(Decision {
        local: place.local,
        definition: definition.location,
        location,
        value,
        target: targets.target_for_value(u128::from(value)),
    })
}

struct Uses<'a> {
    locals: &'a HashSet<mir::Local>,
    writes: &'a HashMap<mir::Location, mir::Local>,
    reads: &'a HashMap<mir::Location, mir::Local>,
    invalid: bool,
}
impl<'tcx> Visitor<'tcx> for Uses<'_> {
    fn visit_local(&mut self, local: mir::Local, context: PlaceContext, location: mir::Location) {
        if !self.locals.contains(&local) {
            return;
        }
        match context {
            PlaceContext::MutatingUse(MutatingUseContext::Store)
                if self.writes.get(&location) == Some(&local) => {}
            PlaceContext::NonMutatingUse(NonMutatingUseContext::Copy)
                if self.reads.get(&location) == Some(&local) => {}
            PlaceContext::NonUse(NonUseContext::StorageLive | NonUseContext::StorageDead) => {}
            _ => self.invalid = true,
        }
    }
}

pub(super) fn validate<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    paths: &[Trace<'_, 'tcx>; 2],
) -> Result<()> {
    let mut reads = HashMap::new();
    let locals: HashSet<_> = paths
        .iter()
        .flat_map(|p| &p.cleanup)
        .map(|d| {
            reads.insert(d.location, d.local);
            d.local
        })
        .collect();
    if locals.len() != 2 || reads.len() != 2 {
        return Err(Error::ControlFlow);
    }
    let mut writes = HashMap::new();
    for (block, data) in body.basic_blocks.iter_enumerated() {
        for (statement_index, statement) in data.statements.iter().enumerate() {
            if let mir::StatementKind::Assign(pair) = &statement.kind
                && locals.contains(&pair.0.local)
            {
                if !pair.0.projection.is_empty() {
                    return Err(Error::Assignment);
                }
                constant(tcx, &pair.1)?;
                writes.insert(
                    mir::Location {
                        block,
                        statement_index,
                    },
                    pair.0.local,
                );
            }
        }
    }
    let mut uses = Uses {
        locals: &locals,
        writes: &writes,
        reads: &reads,
        invalid: false,
    };
    uses.visit_body(body);
    if uses.invalid {
        return Err(Error::Assignment);
    }
    Ok(())
}

pub(in crate::owned_linear) fn assignments(trace: &Trace<'_, '_>) -> HashSet<mir::Location> {
    let locals: HashSet<_> = trace.cleanup.iter().map(|d| d.local).collect();
    trace
        .assignments
        .iter()
        .filter(|a| locals.contains(&a.destination.local))
        .map(|a| a.location)
        .collect()
}
